use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    routing::any,
    Router,
};
use std::env;
use std::sync::Arc;
use tokio::io::{BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{error, info};
use tunnel_protocol::{decode_body, encode_body, read_frame, write_frame, TunnelRequest, TunnelResponse};

/// Shared connection to the active tunnel client
struct TunnelConnection {
    reader: Arc<RwLock<BufReader<ReadHalf<TcpStream>>>>,
    writer: Arc<RwLock<WriteHalf<TcpStream>>>,
}

impl TunnelConnection {
    fn new(stream: TcpStream) -> Self {
        let (read_half, write_half) = tokio::io::split(stream);
        Self {
            reader: Arc::new(RwLock::new(BufReader::new(read_half))),
            writer: Arc::new(RwLock::new(write_half)),
        }
    }
}

/// Application state shared across handlers
#[derive(Clone)]
struct ServerState {
    active_client: Arc<RwLock<Option<Arc<TunnelConnection>>>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            active_client: Arc::new(RwLock::new(None)),
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse configuration from environment variables
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let tunnel_addr = env::var("TUNNEL_ADDR").unwrap_or_else(|_| "0.0.0.0:7000".to_string());

    // Initialize shared state
    let state = ServerState::new();

    // Spawn tunnel listener task
    let tunnel_state = state.clone();
    let tunnel_addr_clone = tunnel_addr.clone();
    tokio::spawn(async move {
        tunnel_listener(tunnel_addr_clone, tunnel_state).await;
    });

    // Build HTTP router
    let app = Router::new()
        .fallback(any(http_handler))
        .with_state(state);

    // Start HTTP server
    info!("Server running - HTTP on {}, Tunnel on {}", http_addr, tunnel_addr);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Listens for tunnel client connections and manages active client
async fn tunnel_listener(addr: String, state: ServerState) {
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Tunnel listener started on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, remote_addr)) => {
                info!("New client connected from {}", remote_addr);

                // Replace old client with new one
                let new_conn = Arc::new(TunnelConnection::new(stream));
                let mut active = state.active_client.write().await;

                if active.is_some() {
                    info!("Replaced old client connection");
                }

                *active = Some(new_conn.clone());
                drop(active); // Release write lock
            }
            Err(e) => {
                error!("Failed to accept tunnel connection: {}", e);
            }
        }
    }
}

/// Handles all HTTP requests by forwarding them through the tunnel
async fn http_handler(
    State(state): State<ServerState>,
    request: Request<Body>,
) -> Response<Body> {
    // Check if client is connected
    let client_lock = state.active_client.read().await;
    let client = match &*client_lock {
        Some(c) => c.clone(),
        None => {
            drop(client_lock);
            return Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(Body::from("No tunnel client connected"))
                .unwrap();
        }
    };
    drop(client_lock);

    // Forward request through tunnel with timeout
    match timeout(
        Duration::from_secs(30),
        forward_request(client.clone(), request)
    ).await {
        Ok(Ok(response)) => response,
        Ok(Err(msg)) => {
            error!("Tunnel error: {}", msg);

            // Clean up broken connection from active client slot
            let mut active = state.active_client.write().await;
            if let Some(current) = &*active {
                if Arc::ptr_eq(current, &client) {
                    info!("Removing broken client connection");
                    *active = None;
                }
            }
            drop(active);

            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(msg))
                .unwrap()
        }
        Err(_) => {
            error!("Tunnel request timeout");

            // Clean up timed-out connection from active client slot
            let mut active = state.active_client.write().await;
            if let Some(current) = &*active {
                if Arc::ptr_eq(current, &client) {
                    info!("Removing timed-out client connection");
                    *active = None;
                }
            }
            drop(active);

            Response::builder()
                .status(StatusCode::GATEWAY_TIMEOUT)
                .body(Body::from("Tunnel request timeout"))
                .unwrap()
        }
    }
}

/// Forwards an HTTP request through the tunnel and returns the response
async fn forward_request(
    client: Arc<TunnelConnection>,
    request: Request<Body>,
) -> Result<Response<Body>, String> {
    // Extract request components
    let method = request.method().to_string();
    let path = request.uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();

    let headers: Vec<(String, String)> = request
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    // Read request body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes.to_vec(),
        Err(e) => return Err(format!("Failed to read request body: {}", e)),
    };

    // Construct tunnel request
    let tunnel_req = TunnelRequest {
        method,
        path,
        headers,
        body: encode_body(&body_bytes),
    };

    // Serialize to JSON
    let payload = match serde_json::to_vec(&tunnel_req) {
        Ok(p) => p,
        Err(e) => return Err(format!("Failed to serialize request: {}", e)),
    };

    // Write to tunnel
    {
        let mut writer = client.writer.write().await;
        if let Err(e) = write_frame(&mut *writer, &payload).await {
            return Err(format!("Tunnel write failed: {}", e));
        }
    }

    // Read response from tunnel
    let response_payload = {
        let mut reader = client.reader.write().await;
        match read_frame(&mut *reader).await {
            Ok(p) => p,
            Err(e) => return Err(format!("Tunnel read failed: {}", e)),
        }
    };

    // Deserialize tunnel response
    let tunnel_resp: TunnelResponse = match serde_json::from_slice(&response_payload) {
        Ok(r) => r,
        Err(e) => return Err(format!("Invalid tunnel response: {}", e)),
    };

    // Decode response body
    let response_body = match decode_body(&tunnel_resp.body) {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to decode response body: {}", e)),
    };

    // Build HTTP response
    let mut response_builder = Response::builder().status(tunnel_resp.status);

    for (name, value) in tunnel_resp.headers {
        response_builder = response_builder.header(name, value);
    }

    Ok(response_builder.body(Body::from(response_body)).unwrap())
}
