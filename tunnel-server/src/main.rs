use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode, header, HeaderMap},
    routing::{any, get},
    Router,
};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use std::env;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::sync::{mpsc, RwLock, oneshot};
use tokio::time::{timeout, Duration};
use tracing::{error, info};
use tunnel_protocol::{decode_body, encode_body, read_frame, write_frame, TunnelRequest, TunnelResponse};

/// Request sent to the tunnel worker
struct TunnelWorkerRequest {
    payload: Vec<u8>,
    response_tx: oneshot::Sender<Result<Vec<u8>, String>>,
}

/// Handle to communicate with the tunnel worker
#[derive(Clone)]
struct TunnelConnection {
    request_tx: mpsc::UnboundedSender<TunnelWorkerRequest>,
}

/// Application state shared across handlers
#[derive(Clone)]
struct ServerState {
    active_client: Arc<RwLock<Option<Arc<TunnelConnection>>>>,
    tunnel_auth: Option<String>, // username:password for Basic Auth
}

impl ServerState {
    fn new(tunnel_auth: Option<String>) -> Self {
        Self {
            active_client: Arc::new(RwLock::new(None)),
            tunnel_auth,
        }
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse configuration from environment variables
    let http_addr = env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let tunnel_auth = env::var("TUNNEL_AUTH").ok();

    // Log authentication status
    if tunnel_auth.is_some() {
        info!("Tunnel authentication enabled");
    } else {
        info!("Tunnel authentication disabled");
    }

    // Initialize shared state
    let state = ServerState::new(tunnel_auth);

    // Build HTTP router
    let app = Router::new()
        .route("/tunnel", get(tunnel_upgrade_handler))
        .fallback(any(http_handler))
        .with_state(state);

    // Start HTTP server
    info!("Server running on {}", http_addr);
    let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Extracts Basic Auth credentials from Authorization header
/// Returns Some(username:password) if valid Basic Auth header is present
fn extract_basic_auth(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(header::AUTHORIZATION)?.to_str().ok()?;

    if !auth_header.starts_with("Basic ") {
        return None;
    }

    let encoded = auth_header.strip_prefix("Basic ")?;
    let decoded = tunnel_protocol::decode_body(encoded).ok()?;
    let credentials = String::from_utf8(decoded).ok()?;

    Some(credentials)
}

/// Handles HTTP Upgrade requests to establish tunnel connections
async fn tunnel_upgrade_handler(
    State(state): State<ServerState>,
    request: Request<Body>,
) -> Response<Body> {
    // Check authentication if enabled
    if let Some(ref expected_auth) = state.tunnel_auth {
        match extract_basic_auth(request.headers()) {
            Some(provided_auth) if provided_auth == *expected_auth => {
                // Authentication successful
                info!("Client authenticated successfully");
            }
            Some(_) => {
                // Invalid credentials
                error!("Authentication failed: Invalid credentials");
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(header::WWW_AUTHENTICATE, "Basic realm=\"tunnel\"")
                    .body(Body::from("Invalid credentials"))
                    .unwrap();
            }
            None => {
                // Missing Authorization header
                error!("Authentication failed: Missing Authorization header");
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header(header::WWW_AUTHENTICATE, "Basic realm=\"tunnel\"")
                    .body(Body::from("Authorization required"))
                    .unwrap();
            }
        }
    }

    // Check for upgrade headers
    let upgrade_header = request.headers().get(header::UPGRADE);
    let connection_header = request.headers().get(header::CONNECTION);

    let is_upgrade = upgrade_header
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("tunnel"))
        .unwrap_or(false);

    let has_upgrade_connection = connection_header
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_lowercase().contains("upgrade"))
        .unwrap_or(false);

    if !is_upgrade || !has_upgrade_connection {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing or invalid Upgrade headers"))
            .unwrap();
    }

    // Attempt to upgrade the connection
    let upgrade_result = hyper::upgrade::on(request);

    // Send 101 Switching Protocols response
    let response = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header(header::UPGRADE, "tunnel")
        .header(header::CONNECTION, "Upgrade")
        .body(Body::empty())
        .unwrap();

    // Spawn task to handle the upgraded connection
    tokio::spawn(async move {
        match upgrade_result.await {
            Ok(upgraded) => {
                info!("Client upgraded to tunnel protocol");

                // Create channel for communicating with worker
                let (request_tx, request_rx) = mpsc::unbounded_channel();

                let new_conn = Arc::new(TunnelConnection { request_tx });

                // Update active client
                let mut active = state.active_client.write().await;
                if active.is_some() {
                    info!("Replaced old client connection");
                }
                *active = Some(new_conn.clone());
                drop(active);

                // Spawn worker to handle the actual I/O
                tunnel_worker(upgraded, request_rx).await;

                // Worker exited, remove from active clients
                let mut active = state.active_client.write().await;
                if let Some(current) = &*active {
                    if Arc::ptr_eq(current, &new_conn) {
                        *active = None;
                        info!("Client disconnected");
                    }
                }
            }
            Err(e) => {
                error!("Failed to upgrade connection: {}", e);
            }
        }
    });

    response
}

/// Worker task that handles I/O for a tunnel connection
async fn tunnel_worker(
    upgraded: Upgraded,
    mut request_rx: mpsc::UnboundedReceiver<TunnelWorkerRequest>,
) {
    let io = TokioIo::new(upgraded);
    let (read_half, write_half) = tokio::io::split(io);
    let mut reader = BufReader::new(read_half);
    let mut writer = write_half;

    while let Some(req) = request_rx.recv().await {
        // Write request to tunnel
        if let Err(e) = write_frame(&mut writer, &req.payload).await {
            let _ = req.response_tx.send(Err(format!("Tunnel write failed: {}", e)));
            break;
        }

        // Read response from tunnel
        match read_frame(&mut reader).await {
            Ok(response_payload) => {
                let _ = req.response_tx.send(Ok(response_payload));
            }
            Err(e) => {
                let _ = req.response_tx.send(Err(format!("Tunnel read failed: {}", e)));
                break;
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

    // Create oneshot channel for response
    let (response_tx, response_rx) = oneshot::channel();

    // Send request to worker
    let worker_req = TunnelWorkerRequest {
        payload,
        response_tx,
    };

    if client.request_tx.send(worker_req).is_err() {
        return Err("Tunnel connection closed".to_string());
    }

    // Wait for response
    let response_payload = match response_rx.await {
        Ok(Ok(payload)) => payload,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err("Tunnel worker disappeared".to_string()),
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
