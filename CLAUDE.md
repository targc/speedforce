# HTTP-over-TCP Tunnel (speedforce)

## Responsibilities

- Provide a simple HTTP tunnel for local development and webhook debugging
- Forward HTTP requests from a public server to local development machines via persistent TCP connections
- Support auto-reconnection and seamless client replacement
- Handle arbitrary HTTP methods, paths, headers, and binary bodies
- Maintain single active client semantics (last client connected wins)

## Tech Stack

- **Language:** Rust 2021 edition
- **Async Runtime:** tokio (recommended for ecosystem compatibility)
- **HTTP Server:** axum or hyper (server-side HTTP endpoint)
- **HTTP Client:** reqwest or hyper (client-side local forwarding)
- **Serialization:** serde + serde_json (protocol messages)
- **Binary Encoding:** base64 crate (for binary body data in JSON)
- **Workspace Structure:** Two binaries in single workspace

## Coding Styles

- **Philosophy:** Lean and simple - avoid over-engineering, MVP-first approach
- **Project Structure:** Flat workspace structure with shared protocol module
- **Error Handling:** Fail fast with clear error messages, no complex retry logic
- **Naming:** Self-documenting names (TunnelRequest, TunnelResponse, etc.)
- **Async:** Use tokio::spawn for concurrent tasks, handle cancellation gracefully
- **Logging:** Use tracing crate for structured logging

## Project Structure

```
speedforce/
├── Cargo.toml              # Workspace manifest
├── tunnel-server/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # Server binary
├── tunnel-client/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs         # Client binary
└── tunnel-protocol/
    ├── Cargo.toml
    └── src/
        └── lib.rs          # Shared protocol definitions
```

## Protocol Specification

### Framing Format

All messages over the TCP connection use length-prefixed framing:

```
[4 bytes: u32 big-endian length] [N bytes: JSON payload]
```

- Length prefix is a 32-bit unsigned integer in **big-endian** byte order
- Length indicates the number of bytes in the JSON payload that follows
- Maximum message size: 2^32 - 1 bytes (no artificial limit imposed)

### Message Types

#### TunnelRequest (Server → Client)

Represents an HTTP request to be forwarded to the local service.

```rust
#[derive(Serialize, Deserialize)]
struct TunnelRequest {
    method: String,           // HTTP method (GET, POST, PUT, DELETE, etc.)
    path: String,             // Full path including query string (e.g., "/api/v1/webhook?x=1")
    headers: Vec<(String, String)>,  // Header name-value pairs
    body: String,             // Base64-encoded body bytes (supports binary data)
}
```

**Example JSON:**
```json
{
  "method": "POST",
  "path": "/webhook?source=github",
  "headers": [
    ["content-type", "application/json"],
    ["user-agent", "GitHub-Hookshot/abc123"]
  ],
  "body": "eyJldmVudCI6InB1c2gifQ=="
}
```

#### TunnelResponse (Client → Server)

Represents the HTTP response from the local service.

```rust
#[derive(Serialize, Deserialize)]
struct TunnelResponse {
    status: u16,              // HTTP status code (200, 404, 500, etc.)
    headers: Vec<(String, String)>,  // Header name-value pairs
    body: String,             // Base64-encoded body bytes (supports binary data)
}
```

**Example JSON:**
```json
{
  "status": 200,
  "headers": [
    ["content-type", "application/json"]
  ],
  "body": "eyJzdWNjZXNzIjp0cnVlfQ=="
}
```

### Binary Data Handling

- HTTP request/response bodies are arbitrary binary data
- Encode body bytes as **base64** before serializing to JSON
- Decode base64 string after deserializing from JSON
- Empty bodies should be encoded as empty string `""`

### Protocol Constraints

- **Sequential processing:** One request-response pair at a time per connection
- **No message IDs:** Request-response pairing is implicit (synchronous)
- **No multiplexing:** Single in-flight request per tunnel connection
- **Path preservation:** The `path` field must include original path AND query string exactly as received

## Tunnel Server (tunnel-server)

### Responsibilities

- Accept incoming HTTP requests on public endpoint
- Maintain single active TCP tunnel connection with client
- Forward HTTP requests to active client via tunnel protocol
- Return tunnel responses to original HTTP callers
- Handle client connect/disconnect/replacement scenarios

### Configuration

Environment variables:

- `HTTP_ADDR` - Address to bind HTTP server (default: `0.0.0.0:8080`)
- `TUNNEL_ADDR` - Address to bind TCP tunnel listener (default: `0.0.0.0:7000`)

### Architecture

The server runs two concurrent tasks:

1. **HTTP Server Task:** Listens for HTTP requests, forwards to tunnel
2. **Tunnel Listener Task:** Accepts client connections, replaces old client

Shared state:
```rust
struct ServerState {
    active_client: Arc<RwLock<Option<TunnelConnection>>>,
}
```

### HTTP Request Handling

**Endpoint:** Accept all methods and paths

**Behavior:**

1. Extract HTTP method, full path (with query string), headers, body bytes
2. Check if active client exists:
   - If **no client connected:** Return 503 Service Unavailable
   - If **client connected:** Proceed to step 3
3. Construct `TunnelRequest` message:
   - method: HTTP method as string
   - path: Full path including query (e.g., `/api/v1?x=1`)
   - headers: Convert to vec of (name, value) tuples
   - body: Base64-encode body bytes
4. Serialize to JSON, write length-prefixed frame to TCP connection
5. Read length-prefixed response frame from TCP connection
6. Deserialize `TunnelResponse` from JSON
7. Base64-decode response body
8. Return HTTP response with status, headers, body from tunnel response

**Error Handling:**

- If no client connected: HTTP 503 with body `"No tunnel client connected"`
- If tunnel write fails: HTTP 502 with body `"Tunnel write failed"`
- If tunnel read fails: HTTP 502 with body `"Tunnel read failed"`
- If deserialization fails: HTTP 502 with body `"Invalid tunnel response"`

**Timeout:** Optional timeout (e.g., 30 seconds) for waiting on tunnel response

### Tunnel Client Management

**Listener:** TCP listener on `TUNNEL_ADDR`

**Behavior:**

1. Accept incoming TCP connections in loop
2. When new client connects:
   - Log: "New client connected from {address}"
   - If active client exists:
     - Close old connection gracefully
     - Log: "Replaced old client connection"
   - Store new connection as active client
3. Spawn task to monitor connection health (detect disconnect)
4. On disconnect:
   - Remove from active client slot
   - Log: "Client disconnected"

**Connection Storage:**

```rust
struct TunnelConnection {
    reader: BufReader<OwnedReadHalf>,  // TCP read half
    writer: OwnedWriteHalf,            // TCP write half
}
```

**Concurrency:** Use `Arc<RwLock<Option<TunnelConnection>>>` for shared state:
- HTTP handlers acquire read lock to check/use active client
- Tunnel listener acquires write lock to replace client

### Startup Sequence

1. Parse environment variables (HTTP_ADDR, TUNNEL_ADDR)
2. Initialize shared state (active_client = None)
3. Spawn tunnel listener task
4. Start HTTP server
5. Log: "Server running - HTTP on {HTTP_ADDR}, Tunnel on {TUNNEL_ADDR}"

## Tunnel Client (tunnel-client)

### Responsibilities

- Maintain persistent TCP connection to server
- Receive tunnel requests from server
- Forward requests to local HTTP service
- Send tunnel responses back to server
- Auto-reconnect on connection failures

### Configuration

Environment variables:

- `SERVER_ADDR` - Server TCP address to connect to (default: `127.0.0.1:7000`)
- `LOCAL_PORT` - Local HTTP service port (default: `3000`)

### Architecture

Single main loop with two phases:

1. **Connection Phase:** Connect to server with exponential backoff
2. **Forwarding Phase:** Process requests until disconnect

### Connection Logic

**Reconnect Loop:**

```rust
loop {
    match connect_to_server(server_addr).await {
        Ok(stream) => {
            log: "Connected to server at {server_addr}";
            handle_tunnel_connection(stream, local_port).await;
            log: "Disconnected from server";
        }
        Err(e) => {
            log: "Connection failed: {e}";
        }
    }

    // Exponential backoff
    sleep(backoff_duration).await;
    backoff_duration = min(backoff_duration * 2, MAX_BACKOFF);
}
```

**Backoff Strategy:**

- Initial delay: 1 second
- Backoff multiplier: 2x
- Maximum delay: 30 seconds
- No maximum retry count (retry forever)

### Request Forwarding Logic

**Main Loop:**

1. Read length-prefixed frame from TCP connection
2. Deserialize `TunnelRequest` from JSON
3. Base64-decode request body
4. Construct local HTTP request:
   - URL: `http://127.0.0.1:{LOCAL_PORT}{path}`
   - Method: From tunnel request
   - Headers: From tunnel request
   - Body: Decoded bytes
5. Execute HTTP request to local service:
   - **Success:** Extract status, headers, body
   - **Failure:** Use status 502, error message in body
6. Construct `TunnelResponse`:
   - status: HTTP status code
   - headers: Response headers as vec
   - body: Base64-encode response body
7. Serialize to JSON, write length-prefixed frame to TCP connection
8. Repeat from step 1

**Error Handling:**

- If local HTTP request fails (connection refused, timeout, etc.):
  - Log: "Local HTTP request failed: {error}"
  - Create error response:
    ```json
    {
      "status": 502,
      "headers": [["content-type", "text/plain"]],
      "body": "<base64 of 'Local service unavailable'>"
    }
    ```
  - Send error response to server
- If TCP read fails: Exit forwarding loop, trigger reconnect
- If TCP write fails: Exit forwarding loop, trigger reconnect

### Startup Sequence

1. Parse environment variables (SERVER_ADDR, LOCAL_PORT)
2. Log: "Starting client - will forward to http://127.0.0.1:{LOCAL_PORT}"
3. Enter reconnect loop
4. Never exit (runs until killed)

## Dependencies

### Workspace Cargo.toml

```toml
[workspace]
members = ["tunnel-server", "tunnel-client", "tunnel-protocol"]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### tunnel-protocol/Cargo.toml

```toml
[package]
name = "tunnel-protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
base64 = { workspace = true }
tokio = { workspace = true }
```

### tunnel-server/Cargo.toml

```toml
[package]
name = "tunnel-server"
version = "0.1.0"
edition = "2021"

[dependencies]
tunnel-protocol = { path = "../tunnel-protocol" }
tokio = { workspace = true }
axum = "0.7"
tower = "0.4"
hyper = "1.0"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

### tunnel-client/Cargo.toml

```toml
[package]
name = "tunnel-client"
version = "0.1.0"
edition = "2021"

[dependencies]
tunnel-protocol = { path = "../tunnel-protocol" }
tokio = { workspace = true }
reqwest = "0.11"
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

## Implementation Details

### Length-Prefixed Framing

**Writing a frame:**

```rust
async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8]
) -> Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}
```

**Reading a frame:**

```rust
async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R
) -> Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}
```

### Base64 Body Encoding

**Encoding:**

```rust
use base64::{Engine as _, engine::general_purpose::STANDARD};

let encoded = STANDARD.encode(&body_bytes);
```

**Decoding:**

```rust
let decoded = STANDARD.decode(&encoded_string)?;
```

### Header Conversion

**HTTP headers to Vec<(String, String)>:**

```rust
let headers: Vec<(String, String)> = http_headers
    .iter()
    .map(|(name, value)| {
        (
            name.as_str().to_string(),
            value.to_str().unwrap_or("").to_string()
        )
    })
    .collect();
```

### Path Preservation

**Extracting full path with query:**

```rust
// axum example
let uri = request.uri();
let path = uri.path_and_query()
    .map(|pq| pq.as_str())
    .unwrap_or("/");
```

This ensures `/api/v1?x=1&y=2` is preserved exactly.

### Concurrent Client Replacement

**Pattern for safe replacement:**

```rust
async fn handle_new_client(
    state: Arc<ServerState>,
    new_conn: TcpStream
) {
    let mut active = state.active_client.write().await;

    // Close old connection if exists
    if let Some(old) = active.take() {
        drop(old); // Closes TCP connection
        tracing::info!("Replaced old client");
    }

    // Store new connection
    *active = Some(TunnelConnection::new(new_conn));
    tracing::info!("New client active");
}
```

### HTTP Request Timeout

Use `tokio::time::timeout` to avoid hanging indefinitely:

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    forward_request_through_tunnel(req)
).await;

match result {
    Ok(Ok(response)) => Ok(response),
    Ok(Err(e)) => Err(e),
    Err(_) => Err(Error::Timeout),
}
```

## Testing

### Manual Testing Checklist

#### Setup

Start a simple local HTTP server for testing:

```bash
# Terminal 1: Simple test server on port 3000
python3 -m http.server 3000
```

Or use a minimal echo server:

```bash
# Using netcat (returns fixed response)
while true; do echo -e "HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nSuccess" | nc -l 3000; done
```

#### Test Case 1: Basic Forwarding

**Setup:**
```bash
# Terminal 1: Start server
cd tunnel-server
SERVER_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 cargo run

# Terminal 2: Start client
cd tunnel-client
SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 cargo run

# Terminal 3: Local test service
python3 -m http.server 3000
```

**Test:**
```bash
# Terminal 4: Send HTTP request to server
curl -v http://localhost:8080/

# Expected: Response from local python server
```

**Validation:**
- [ ] HTTP request reaches local service
- [ ] Response is returned to curl
- [ ] Server logs show request forwarding
- [ ] Client logs show local HTTP request

#### Test Case 2: Path and Query Preservation

**Test:**
```bash
curl -v "http://localhost:8080/api/v1/webhook?source=test&id=123"
```

**Validation:**
- [ ] Local service receives exact path: `/api/v1/webhook?source=test&id=123`
- [ ] Query parameters are preserved
- [ ] No URL encoding issues

#### Test Case 3: POST with Body

**Test:**
```bash
curl -v -X POST \
  -H "Content-Type: application/json" \
  -d '{"event":"push","repo":"test"}' \
  http://localhost:8080/webhook
```

**Validation:**
- [ ] POST method is forwarded correctly
- [ ] JSON body arrives intact at local service
- [ ] Content-Type header is preserved
- [ ] Response is returned successfully

#### Test Case 4: Client Not Connected (503)

**Setup:**
```bash
# Start only the server (no client)
cd tunnel-server
cargo run
```

**Test:**
```bash
curl -v http://localhost:8080/
```

**Expected:**
- HTTP status: 503 Service Unavailable
- Body: "No tunnel client connected"

**Validation:**
- [ ] Server returns 503 immediately
- [ ] Error message is clear

#### Test Case 5: Client Reconnection

**Setup:**
```bash
# Server and client running, local service running
```

**Test:**
```bash
# Terminal 1: Send successful request
curl http://localhost:8080/test1
# Should succeed

# Terminal 2: Kill the client (Ctrl+C)
# Wait 2 seconds

# Terminal 1: Send request while client is down
curl http://localhost:8080/test2
# Should get 503

# Terminal 2: Restart client
cargo run

# Terminal 1: Send request after client reconnects
curl http://localhost:8080/test3
# Should succeed again
```

**Validation:**
- [ ] First request succeeds
- [ ] Request during downtime returns 503
- [ ] Client reconnects automatically
- [ ] Requests succeed after reconnection
- [ ] No server restart needed

#### Test Case 6: Multiple Clients (Last One Wins)

**Setup:**
```bash
# Start server
cd tunnel-server
cargo run
```

**Test:**
```bash
# Terminal 1: Start first client with LOCAL_PORT=3000
cd tunnel-client
LOCAL_PORT=3000 cargo run

# Terminal 2: Start second client with LOCAL_PORT=3001
cd tunnel-client
LOCAL_PORT=3001 cargo run

# Wait a moment for second client to connect

# Terminal 3: Send request
curl http://localhost:8080/
```

**Validation:**
- [ ] Server logs show "Replaced old client"
- [ ] First client connection is closed
- [ ] Request is forwarded to second client (port 3001)
- [ ] No requests reach first client

#### Test Case 7: Binary Data Handling

**Setup:**
```bash
# Create a binary file
dd if=/dev/urandom of=/tmp/test.bin bs=1024 count=10
```

**Test:**
```bash
curl -v -X POST \
  -H "Content-Type: application/octet-stream" \
  --data-binary @/tmp/test.bin \
  http://localhost:8080/upload
```

**Validation:**
- [ ] Binary data is transmitted without corruption
- [ ] Base64 encoding/decoding works correctly
- [ ] Content-Type header preserved
- [ ] Response body is binary-safe

#### Test Case 8: Server Restart Recovery

**Test:**
```bash
# Client running, server running
curl http://localhost:8080/  # Succeeds

# Stop server (Ctrl+C)
# Client logs should show connection errors and reconnection attempts

# Restart server
cargo run

# Client should reconnect automatically (check logs)
curl http://localhost:8080/  # Should succeed again
```

**Validation:**
- [ ] Client detects server disconnect
- [ ] Client enters reconnection loop
- [ ] Client successfully reconnects when server is back
- [ ] Requests work after server restart

### Integration Testing

**End-to-End Webhook Scenario:**

1. Deploy server on public VPS (e.g., DigitalOcean)
2. Run client on local dev machine
3. Configure GitHub webhook to point to `http://<VPS_IP>:8080/webhook`
4. Run local webhook handler on port 3000
5. Trigger GitHub event (push, PR, etc.)
6. Verify webhook is received by local handler

**Expected Flow:**
```
GitHub → VPS (tunnel-server:8080) → TCP tunnel →
Local machine (tunnel-client) → localhost:3000 (webhook handler)
```

## Common Issues & Debugging

### Issue: Connection Refused on Client

**Symptom:** Client logs show "Connection refused" repeatedly

**Debugging:**
1. Check server is running: `netstat -an | grep 7000`
2. Check firewall rules: `sudo ufw status`
3. Verify SERVER_ADDR environment variable
4. Check server logs for tunnel listener startup

**Solution:**
- Ensure server is running and tunnel listener started
- If server is remote, check firewall allows port 7000
- Verify network connectivity: `telnet <SERVER_IP> 7000`

### Issue: 502 Bad Gateway

**Symptom:** HTTP requests return 502 from server

**Possible Causes:**
1. Local service is down (client can't reach localhost:3000)
2. Local service crashes during request processing
3. Tunnel protocol deserialization error

**Debugging:**
```bash
# Check local service is running
curl http://127.0.0.1:3000/

# Check client logs for HTTP request errors
# Check server logs for tunnel communication errors
```

**Solution:**
- Ensure local service is running and responding
- Check local service logs for errors
- Verify LOCAL_PORT configuration

### Issue: Requests Timing Out

**Symptom:** Curl hangs, eventually times out

**Possible Causes:**
1. Deadlock in tunnel protocol (unlikely with sequential processing)
2. Local service hangs without responding
3. Network issue between server and client

**Debugging:**
```bash
# Enable verbose logging
RUST_LOG=debug cargo run

# Test local service directly
curl -v http://127.0.0.1:3000/

# Check TCP connection state
netstat -an | grep 7000
```

**Solution:**
- Implement timeout on server side (30s recommended)
- Fix hanging local service
- Check network stability

### Issue: Body Corruption

**Symptom:** Binary data is corrupted or truncated

**Debugging:**
1. Check Content-Length headers
2. Verify base64 encoding/decoding
3. Test with known binary file

**Solution:**
- Ensure base64 encoding uses STANDARD (not URL_SAFE)
- Verify no string conversions assume UTF-8
- Check framing length prefix is correct

### Issue: Client Won't Reconnect After Network Change

**Symptom:** Client stops reconnecting after WiFi change or network interruption

**Debugging:**
```bash
# Check DNS resolution
nslookup <SERVER_ADDR>

# Check routing
traceroute <SERVER_ADDR>

# Test direct connectivity
nc -zv <SERVER_IP> 7000
```

**Solution:**
- Ensure reconnect loop continues indefinitely
- Check exponential backoff doesn't exceed reasonable maximum
- Verify no early exit on specific error types

## MVP Limitations

The following features are **explicitly NOT implemented** in this MVP:

- **TLS/Encryption:** All traffic is plaintext TCP and HTTP
  - Workaround: Use SSH tunnel or VPN for secure communication
  - Future: Add TLS support with certificates

- **Authentication:** No client authentication or authorization
  - Workaround: Use firewall rules to restrict TCP port access
  - Future: Add client tokens or mutual TLS

- **Multi-Tenancy:** Only one client can be active at a time
  - Workaround: Run multiple server instances on different ports
  - Future: Add client IDs and route by subdomain/path prefix

- **Request Queueing:** No persistent queue if client is disconnected
  - Workaround: Third-party webhook providers usually retry automatically
  - Future: Add Redis-backed request queue

- **Multiplexing:** Only one in-flight request per connection
  - Impact: High-latency requests block subsequent requests
  - Workaround: Acceptable for typical webhook use cases
  - Future: Add message IDs and concurrent request handling

- **Load Balancing:** No distribution across multiple clients
  - Workaround: Single client is sufficient for dev/debug use case
  - Future: Add client pools with round-robin

- **Metrics/Monitoring:** No Prometheus metrics or health endpoints
  - Workaround: Use log analysis and `netstat` for debugging
  - Future: Add `/metrics` and `/health` endpoints

- **Request/Response Size Limits:** No enforced maximum message size
  - Risk: Large requests could cause memory issues
  - Workaround: Don't use for large file uploads
  - Future: Add configurable size limits (e.g., 10MB)

- **WebSocket Support:** Only HTTP request/response, no streaming
  - Workaround: Not supported, use separate solution
  - Future: Add WebSocket tunneling support

- **Custom Error Pages:** Generic error messages only
  - Workaround: Acceptable for dev debugging
  - Future: Add configurable error page templates

## Acceptance Criteria

The implementation is complete when ALL of these scenarios pass:

### 1. Basic Forwarding
- [x] Server running on 0.0.0.0:8080 (HTTP) and 0.0.0.0:7000 (TCP)
- [x] Client connected to server and forwarding to localhost:3000
- [x] HTTP request to server is forwarded to local service
- [x] Local service response is returned to original caller
- [x] Method, path, headers, body are preserved exactly

### 2. Arbitrary Paths and Queries
- [x] Request to `/foo/bar?x=1&y=2` arrives at local service as `/foo/bar?x=1&y=2`
- [x] Special characters in query strings are preserved
- [x] POST requests with JSON bodies work correctly

### 3. Client Down (503)
- [x] When no client connected, server returns 503 Service Unavailable
- [x] Error message is clear and helpful
- [x] Server continues running and accepting HTTP requests

### 4. Client Reconnect
- [x] Client can be stopped and restarted
- [x] Client reconnects automatically without manual intervention
- [x] After reconnection, requests flow normally
- [x] Server does NOT need to be restarted

### 5. Two Clients (Last One Wins)
- [x] When second client connects, first client is disconnected
- [x] Subsequent requests go only to the latest connected client
- [x] No split-brain behavior (requests never go to old client)
- [x] Server logs indicate client replacement

### 6. Server Restart Recovery
- [x] Client survives server restarts
- [x] Client reconnects when server is back up
- [x] Exponential backoff prevents connection spam
- [x] Maximum backoff prevents excessive delays

### 7. Binary Data Support
- [x] Binary request bodies (octet-stream) are transmitted correctly
- [x] Binary response bodies are transmitted correctly
- [x] No data corruption or truncation
- [x] Base64 encoding/decoding is transparent

### 8. Error Handling
- [x] Local service down: Client returns 502 in tunnel response
- [x] Tunnel write failure: Server returns 502 to HTTP caller
- [x] Invalid protocol message: Server returns 502 to HTTP caller
- [x] All errors are logged with sufficient detail

## Environment Variables Reference

### tunnel-server

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| HTTP_ADDR | HTTP server bind address | 0.0.0.0:8080 | No |
| TUNNEL_ADDR | TCP tunnel listener bind address | 0.0.0.0:7000 | No |

**Example:**
```bash
HTTP_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 cargo run --bin tunnel-server
```

### tunnel-client

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| SERVER_ADDR | Server TCP address to connect to | 127.0.0.1:7000 | No |
| LOCAL_PORT | Local HTTP service port | 3000 | No |

**Example:**
```bash
SERVER_ADDR=example.com:7000 LOCAL_PORT=3000 cargo run --bin tunnel-client
```

## Docker Support

### Dockerfile (tunnel-server)

```dockerfile
FROM rust:1.75 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release --bin tunnel-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tunnel-server /usr/local/bin/

ENV HTTP_ADDR=0.0.0.0:8080
ENV TUNNEL_ADDR=0.0.0.0:7000

EXPOSE 8080 7000

CMD ["tunnel-server"]
```

### Dockerfile (tunnel-client)

```dockerfile
FROM rust:1.75 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release --bin tunnel-client

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tunnel-client /usr/local/bin/

ENV SERVER_ADDR=127.0.0.1:7000
ENV LOCAL_PORT=3000

CMD ["tunnel-client"]
```

### docker-compose.yml

```yaml
version: '3.8'

services:
  tunnel-server:
    build:
      context: .
      dockerfile: Dockerfile.server
    container_name: speedforce-server
    ports:
      - "8080:8080"  # HTTP endpoint
      - "7000:7000"  # Tunnel TCP port
    environment:
      HTTP_ADDR: "0.0.0.0:8080"
      TUNNEL_ADDR: "0.0.0.0:7000"
    networks:
      - speedforce
    restart: unless-stopped

  tunnel-client:
    build:
      context: .
      dockerfile: Dockerfile.client
    container_name: speedforce-client
    environment:
      SERVER_ADDR: "tunnel-server:7000"
      LOCAL_PORT: "3000"
    networks:
      - speedforce
    depends_on:
      - tunnel-server
    restart: unless-stopped

networks:
  speedforce:
    driver: bridge
```

**Notes:**
- Server must be publicly accessible on port 8080 (HTTP) and 7000 (TCP)
- Client typically runs on dev machine (not in Docker)
- Docker networking: Use `host.docker.internal` to reach host machine from container

## Security Considerations

This MVP has NO built-in security features. Consider these risks:

1. **Plaintext Traffic:** All HTTP and TCP traffic is unencrypted
   - Mitigation: Use SSH tunnel, VPN, or deploy on private network

2. **No Authentication:** Anyone who can reach port 7000 can become a tunnel client
   - Mitigation: Firewall rules, VPN, or deploy on trusted network

3. **No Rate Limiting:** Server will forward all HTTP requests to client
   - Mitigation: Use reverse proxy (nginx) with rate limiting

4. **Client Replacement:** Malicious client can hijack tunnel by connecting
   - Mitigation: Firewall rules to restrict TCP port access

5. **Resource Exhaustion:** Large requests could exhaust memory
   - Mitigation: Deploy with resource limits (Docker memory limits)

6. **Log Injection:** HTTP headers/paths are logged as-is
   - Mitigation: Use structured logging (tracing crate helps here)

For production use, implement TLS, authentication, and rate limiting.

## Performance Characteristics

Expected performance for typical webhook use cases:

- **Throughput:** 10-100 requests/second (sequential processing)
- **Latency:** Base latency + network RTT + local service latency
  - Base overhead: ~1-5ms (serialization, framing)
  - Network RTT: Varies (LAN: <1ms, Internet: 10-100ms)
  - Local service: Depends on application
- **Memory:** ~10MB baseline + buffer for request/response bodies
- **CPU:** Minimal (mostly I/O bound)

**Bottlenecks:**

- Sequential request processing (one at a time)
- Network latency between server and client
- Local service response time

**Acceptable Use Cases:**

- GitHub/GitLab webhooks (sporadic, small payloads)
- Stripe/payment webhooks (low frequency)
- CI/CD callbacks (occasional)
- Development/debugging scenarios

**NOT Recommended For:**

- High-frequency webhooks (>100/sec)
- Large file uploads (>10MB)
- Real-time streaming
- Production traffic

## Logging

Use structured logging with the `tracing` crate:

**Server logs:**
- `info`: Server startup, client connections, client replacements
- `debug`: Each HTTP request (method, path), tunnel operations
- `error`: Connection failures, protocol errors

**Client logs:**
- `info`: Connection established, disconnected, reconnecting
- `debug`: Each tunnel request received, local HTTP request made
- `error`: Connection failures, HTTP errors

**Example output:**
```
2024-01-15T10:30:00Z INFO tunnel_server: Server started http_addr=0.0.0.0:8080 tunnel_addr=0.0.0.0:7000
2024-01-15T10:30:05Z INFO tunnel_server: New client connected remote_addr=192.168.1.100:54321
2024-01-15T10:30:10Z DEBUG tunnel_server: HTTP request received method=POST path=/webhook
2024-01-15T10:30:10Z DEBUG tunnel_client: Forwarding to local service url=http://127.0.0.1:3000/webhook
2024-01-15T10:30:11Z DEBUG tunnel_client: Local service responded status=200
```

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

## Data Flow Diagram

```
┌─────────────┐                 ┌──────────────┐                ┌──────────────┐               ┌─────────────┐
│   External  │                 │    Tunnel    │                │    Tunnel    │               │    Local    │
│   HTTP      │                 │    Server    │                │    Client    │               │   HTTP      │
│   Client    │                 │              │                │              │               │   Service   │
└──────┬──────┘                 └──────┬───────┘                └──────┬───────┘               └──────┬──────┘
       │                               │                               │                              │
       │  1. HTTP Request              │                               │                              │
       │  POST /webhook                │                               │                              │
       ├──────────────────────────────>│                               │                              │
       │                               │                               │                              │
       │                               │  2. Serialize to TunnelRequest│                              │
       │                               │  {method, path, headers, body}│                              │
       │                               │                               │                              │
       │                               │  3. Length-prefixed frame     │                              │
       │                               │  [u32 len][JSON payload]      │                              │
       │                               ├──────────────────────────────>│                              │
       │                               │                               │                              │
       │                               │                               │  4. Deserialize, decode base64
       │                               │                               │                              │
       │                               │                               │  5. HTTP Request             │
       │                               │                               │  POST /webhook               │
       │                               │                               ├─────────────────────────────>│
       │                               │                               │                              │
       │                               │                               │  6. HTTP Response            │
       │                               │                               │  200 OK {json}               │
       │                               │                               │<─────────────────────────────┤
       │                               │                               │                              │
       │                               │  7. Serialize to TunnelResponse                              │
       │                               │  {status, headers, body}      │                              │
       │                               │                               │                              │
       │                               │  8. Length-prefixed frame     │                              │
       │                               │  [u32 len][JSON payload]      │                              │
       │                               │<──────────────────────────────┤                              │
       │                               │                               │                              │
       │  9. Deserialize, decode base64│                               │                              │
       │                               │                               │                              │
       │  10. HTTP Response            │                               │                              │
       │  200 OK {json}                │                               │                              │
       │<──────────────────────────────┤                               │                              │
       │                               │                               │                              │
```

**Key Points:**

1. External HTTP client sends request to server's public HTTP endpoint
2. Server serializes request to TunnelRequest (base64-encoded body)
3. Server sends length-prefixed JSON frame over TCP connection
4. Client receives frame, deserializes, and decodes base64 body
5. Client makes HTTP request to local service (localhost:3000)
6. Local service responds with HTTP response
7. Client serializes response to TunnelResponse (base64-encoded body)
8. Client sends length-prefixed JSON frame back over TCP
9. Server receives frame, deserializes, and decodes base64 body
10. Server returns HTTP response to original external caller

This completes one request-response cycle. The TCP connection remains open for subsequent requests.

---

**Next Steps After Specification:**

Once this specification is approved, proceed to implementation:

1. Create workspace structure with three crates
2. Implement tunnel-protocol with TunnelRequest/TunnelResponse structs
3. Implement tunnel-server with HTTP and TCP listeners
4. Implement tunnel-client with reconnection loop
5. Test against all acceptance criteria
6. Build Docker images and test containerized deployment

This specification is implementation-ready. Developers can now build the system without additional clarification.
