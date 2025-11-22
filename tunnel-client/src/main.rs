use std::env;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{error, info};
use tunnel_protocol::{decode_body, encode_body, read_frame, write_frame, TunnelRequest, TunnelResponse};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse configuration from environment variables
    let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:7000".to_string());
    let local_port = env::var("LOCAL_PORT").unwrap_or_else(|_| "3000".to_string());

    info!("Starting client - will forward to http://127.0.0.1:{}", local_port);

    // Connection loop with exponential backoff
    let mut backoff_duration = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match TcpStream::connect(&server_addr).await {
            Ok(stream) => {
                info!("Connected to server at {}", server_addr);

                // Reset backoff on successful connection
                backoff_duration = Duration::from_secs(1);

                // Handle tunnel connection
                handle_tunnel_connection(stream, &local_port).await;

                info!("Disconnected from server");
            }
            Err(e) => {
                error!("Connection failed: {}", e);
            }
        }

        // Exponential backoff
        info!("Reconnecting in {:?}...", backoff_duration);
        sleep(backoff_duration).await;
        backoff_duration = std::cmp::min(backoff_duration * 2, max_backoff);
    }
}

/// Handles the tunnel connection by processing requests until disconnect
async fn handle_tunnel_connection(stream: TcpStream, local_port: &str) {
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = BufReader::new(read_half);
    let mut writer = write_half;

    loop {
        // Read tunnel request
        let request_payload = match read_frame(&mut reader).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to read frame: {}", e);
                break;
            }
        };

        // Deserialize tunnel request
        let tunnel_req: TunnelRequest = match serde_json::from_slice(&request_payload) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to deserialize request: {}", e);
                break;
            }
        };

        // Process request and send response
        let tunnel_resp = process_request(tunnel_req, local_port).await;

        // Serialize tunnel response
        let response_payload = match serde_json::to_vec(&tunnel_resp) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to serialize response: {}", e);
                break;
            }
        };

        // Write tunnel response
        if let Err(e) = write_frame(&mut writer, &response_payload).await {
            error!("Failed to write frame: {}", e);
            break;
        }
    }
}

/// Processes a tunnel request by forwarding to local HTTP service
async fn process_request(tunnel_req: TunnelRequest, local_port: &str) -> TunnelResponse {
    // Decode request body
    let request_body = match decode_body(&tunnel_req.body) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to decode request body: {}", e);
            return error_response("Failed to decode request body");
        }
    };

    // Build local URL
    let url = format!("http://127.0.0.1:{}{}", local_port, tunnel_req.path);

    // Build HTTP client request
    let client = reqwest::Client::new();
    let mut req_builder = match tunnel_req.method.as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        "HEAD" => client.head(&url),
        "OPTIONS" => client.request(reqwest::Method::OPTIONS, &url),
        other => client.request(reqwest::Method::from_bytes(other.as_bytes()).unwrap_or(reqwest::Method::GET), &url),
    };

    // Add headers
    for (name, value) in tunnel_req.headers {
        req_builder = req_builder.header(name, value);
    }

    // Add body
    req_builder = req_builder.body(request_body);

    // Execute request
    match req_builder.send().await {
        Ok(response) => {
            let status = response.status().as_u16();

            // Extract headers
            let headers: Vec<(String, String)> = response
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.as_str().to_string(),
                        value.to_str().unwrap_or("").to_string(),
                    )
                })
                .collect();

            // Read response body
            let response_body = match response.bytes().await {
                Ok(bytes) => bytes.to_vec(),
                Err(e) => {
                    error!("Failed to read response body: {}", e);
                    return error_response("Failed to read response body");
                }
            };

            TunnelResponse {
                status,
                headers,
                body: encode_body(&response_body),
            }
        }
        Err(e) => {
            error!("Local HTTP request failed: {}", e);
            error_response("Local service unavailable")
        }
    }
}

/// Creates an error response for tunnel communication
fn error_response(message: &str) -> TunnelResponse {
    TunnelResponse {
        status: 502,
        headers: vec![("content-type".to_string(), "text/plain".to_string())],
        body: encode_body(message.as_bytes()),
    }
}
