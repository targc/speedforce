use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_rustls::TlsConnector;
use rustls::{ClientConfig, RootCertStore};
use rustls::pki_types::ServerName;
use tracing::{error, info};
use tunnel_protocol::{decode_body, encode_body, read_frame, write_frame, TunnelRequest, TunnelResponse};

/// Configuration for server connection
struct ServerConfig {
    addr: String,        // Host:port for TCP connection
    use_tls: bool,       // Whether to use TLS
    hostname: String,    // Hostname for SNI and Host header
    auth: Option<String>, // Basic Auth credentials in "username:password" format
    local_port: u16,     // Local service port
}

/// Parses server address from environment variable
/// Supports: https://host, https://host:port, http://host:port, host:port
fn parse_server_addr(addr: &str, auth: Option<String>, local_port: u16) -> Result<ServerConfig, String> {
    if addr.starts_with("https://") {
        let without_protocol = addr.strip_prefix("https://").unwrap();
        let (host, port) = parse_host_port(without_protocol, 443)?;
        Ok(ServerConfig {
            addr: format!("{}:{}", host, port),
            use_tls: true,
            hostname: host,
            auth,
            local_port,
        })
    } else if addr.starts_with("http://") {
        let without_protocol = addr.strip_prefix("http://").unwrap();
        let (host, port) = parse_host_port(without_protocol, 80)?;
        Ok(ServerConfig {
            addr: format!("{}:{}", host, port),
            use_tls: false,
            hostname: host,
            auth,
            local_port,
        })
    } else {
        // Backward compatibility: no protocol means plain TCP
        let (host, port) = parse_host_port(addr, 7000)?;
        Ok(ServerConfig {
            addr: format!("{}:{}", host, port),
            use_tls: false,
            hostname: host,
            auth,
            local_port,
        })
    }
}

/// Parses host and port from address string
fn parse_host_port(addr: &str, default_port: u16) -> Result<(String, u16), String> {
    // Remove trailing slash if present
    let addr = addr.trim_end_matches('/');

    if let Some(colon_pos) = addr.rfind(':') {
        // Check if this is an IPv6 address
        if addr.starts_with('[') {
            // IPv6 format: [host]:port or [host]
            if let Some(bracket_pos) = addr.find(']') {
                let host = addr[1..bracket_pos].to_string();
                if colon_pos > bracket_pos {
                    // Has port
                    let port_str = &addr[colon_pos + 1..];
                    let port = port_str.parse::<u16>()
                        .map_err(|_| format!("Invalid port: {}", port_str))?;
                    Ok((host, port))
                } else {
                    // No port
                    Ok((host, default_port))
                }
            } else {
                Err("Invalid IPv6 address format".to_string())
            }
        } else {
            // IPv4 or hostname: host:port
            let host = addr[..colon_pos].to_string();
            let port_str = &addr[colon_pos + 1..];
            let port = port_str.parse::<u16>()
                .map_err(|_| format!("Invalid port: {}", port_str))?;
            Ok((host, port))
        }
    } else {
        // No port specified, use default
        Ok((addr.to_string(), default_port))
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse configuration from environment variables
    let server_addr_str = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:7000".to_string());
    let local_port_str = env::var("LOCAL_PORT").unwrap_or_else(|_| "3000".to_string());
    let tunnel_auth = env::var("TUNNEL_AUTH").ok();

    // Parse local port
    let local_port = match local_port_str.parse::<u16>() {
        Ok(port) => port,
        Err(e) => {
            error!("Invalid LOCAL_PORT: {}", e);
            return;
        }
    };

    // Validate auth format if provided
    if let Some(ref auth) = tunnel_auth {
        if !auth.contains(':') {
            error!("TUNNEL_AUTH must be in format 'username:password'");
            return;
        }
        info!("Basic authentication enabled");
    } else {
        info!("No authentication configured");
    }

    // Parse server address
    let server_config = match parse_server_addr(&server_addr_str, tunnel_auth, local_port) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to parse SERVER_ADDR: {}", e);
            return;
        }
    };

    info!(
        "Starting client - will connect to {} (TLS: {}) and forward to http://127.0.0.1:{}",
        server_config.addr, server_config.use_tls, server_config.local_port
    );

    // Connection loop with exponential backoff
    let mut backoff_duration = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        match connect_and_upgrade(&server_config).await {
            Ok(stream) => {
                info!("Connected and upgraded to tunnel protocol");

                // Reset backoff on successful connection
                backoff_duration = Duration::from_secs(1);

                // Handle tunnel connection
                handle_tunnel_connection(stream, server_config.local_port).await;

                info!("Disconnected from server");
            }
            Err(e) => {
                error!("Connection/upgrade failed: {}", e);
            }
        }

        // Exponential backoff
        info!("Reconnecting in {:?}...", backoff_duration);
        sleep(backoff_duration).await;
        backoff_duration = std::cmp::min(backoff_duration * 2, max_backoff);
    }
}

/// Creates a TLS connector with system root certificates
fn create_tls_connector() -> Result<TlsConnector, String> {
    let mut root_store = RootCertStore::empty();

    // Add system root certificates
    for cert in webpki_roots::TLS_SERVER_ROOTS.iter() {
        root_store.roots.push(cert.clone());
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

/// Stream type that can be either TLS or plain TCP
enum TunnelStream {
    Tls(tokio_rustls::client::TlsStream<TcpStream>),
    Plain(TcpStream),
}

impl tokio::io::AsyncRead for TunnelStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Tls(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            TunnelStream::Plain(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for TunnelStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            TunnelStream::Tls(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            TunnelStream::Plain(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Tls(s) => std::pin::Pin::new(s).poll_flush(cx),
            TunnelStream::Plain(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            TunnelStream::Tls(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            TunnelStream::Plain(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Sends HTTP Upgrade request over any stream type
async fn send_upgrade_request<S: AsyncReadExt + AsyncWriteExt + Unpin>(
    stream: &mut S,
    hostname: &str,
    auth: Option<&str>,
) -> Result<(), String> {
    // Build Authorization header if credentials provided
    let auth_header = if let Some(credentials) = auth {
        let encoded = encode_body(credentials.as_bytes());
        Some(format!("Authorization: Basic {}\r\n", encoded))
    } else {
        None
    };

    // Send HTTP Upgrade request
    let mut upgrade_request = format!(
        "GET /tunnel HTTP/1.1\r\n\
         Host: {}\r\n\
         Upgrade: tunnel\r\n\
         Connection: Upgrade\r\n",
        hostname
    );

    // Add Authorization header if present
    if let Some(auth) = auth_header {
        upgrade_request.push_str(&auth);
    }

    // End of headers
    upgrade_request.push_str("\r\n");

    stream.write_all(upgrade_request.as_bytes()).await
        .map_err(|e| format!("Failed to send upgrade request: {}", e))?;
    stream.flush().await
        .map_err(|e| format!("Failed to flush upgrade request: {}", e))?;

    // Read HTTP response
    let mut response_buffer = vec![0u8; 1024];
    let mut total_read = 0;

    // Read until we have the complete response headers (ending with \r\n\r\n)
    loop {
        let n = stream.read(&mut response_buffer[total_read..]).await
            .map_err(|e| format!("Failed to read upgrade response: {}", e))?;

        if n == 0 {
            return Err("Connection closed before receiving upgrade response".to_string());
        }

        total_read += n;

        // Check if we have the end of headers
        if total_read >= 4 {
            let headers_end = response_buffer[..total_read]
                .windows(4)
                .position(|window| window == b"\r\n\r\n");

            if headers_end.is_some() {
                break;
            }
        }

        if total_read >= response_buffer.len() {
            return Err("Response headers too large".to_string());
        }
    }

    // Parse the HTTP response status line
    let response_str = String::from_utf8_lossy(&response_buffer[..total_read]);
    let first_line = response_str.lines().next()
        .ok_or("Empty response")?;

    // Check for authentication failure
    if first_line.contains("401") {
        return Err("Authentication failed: Invalid credentials".to_string());
    }

    // Check for 101 Switching Protocols
    if !first_line.contains("101") {
        return Err(format!("Upgrade failed: {}", first_line));
    }

    // Verify Upgrade and Connection headers
    let has_upgrade = response_str.to_lowercase().contains("upgrade: tunnel");
    let has_connection = response_str.to_lowercase().contains("connection: upgrade");

    if !has_upgrade || !has_connection {
        return Err("Missing required upgrade headers in response".to_string());
    }

    info!("HTTP Upgrade successful");
    Ok(())
}

/// Connects to the server and performs HTTP Upgrade handshake
async fn connect_and_upgrade(config: &ServerConfig) -> Result<TunnelStream, String> {
    // Connect TCP
    let tcp_stream = TcpStream::connect(&config.addr).await
        .map_err(|e| format!("TCP connection to {} failed: {}", config.addr, e))?;

    info!("TCP connection established to {}", config.addr);

    if config.use_tls {
        // Establish TLS connection
        info!("Establishing TLS connection to {}", config.hostname);

        let tls_connector = create_tls_connector()
            .map_err(|e| format!("Failed to create TLS connector: {}", e))?;

        let server_name = ServerName::try_from(config.hostname.clone())
            .map_err(|e| format!("Invalid hostname for SNI: {}", e))?;

        let mut tls_stream = tls_connector.connect(server_name, tcp_stream).await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;

        info!("TLS connection established");

        // Send HTTP Upgrade over TLS
        send_upgrade_request(
            &mut tls_stream,
            &config.hostname,
            config.auth.as_deref()
        ).await?;

        Ok(TunnelStream::Tls(tls_stream))
    } else {
        // Plain TCP connection
        let mut tcp_stream = tcp_stream;

        // Send HTTP Upgrade over plain TCP
        send_upgrade_request(
            &mut tcp_stream,
            &config.hostname,
            config.auth.as_deref()
        ).await?;

        Ok(TunnelStream::Plain(tcp_stream))
    }
}

/// Handles the tunnel connection by processing requests until disconnect
async fn handle_tunnel_connection(stream: TunnelStream, local_port: u16) {
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
async fn process_request(tunnel_req: TunnelRequest, local_port: u16) -> TunnelResponse {
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
