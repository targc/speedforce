# speedforce - HTTP-over-TCP Tunnel

A minimal HTTP tunnel for local development and webhook debugging. Forward HTTP requests from a public server to your local development machine via HTTP Upgrade protocol.

## Features

- 🚀 **Simple Setup** - Two binaries, single port, no complex configuration
- 🔄 **Auto-Reconnect** - Client automatically reconnects with exponential backoff
- 🎯 **Path Preservation** - Full URL paths and query strings preserved exactly
- 📦 **Binary Support** - Handles arbitrary binary HTTP bodies via base64 encoding
- 🔌 **Single Port** - HTTP and tunnel traffic multiplexed on one port via HTTP Upgrade
- 🔗 **Standard Protocol** - Uses HTTP 101 Switching Protocols (like WebSocket)
- 🔒 **TLS/HTTPS Support** - Secure encrypted connections with certificate validation
- 🔐 **Basic Authentication** - Optional username/password protection for tunnel connections
- 🐳 **Docker Ready** - Dockerfiles and docker-compose included

## Quick Start

### Binary Installation

**1. Build from source:**
```bash
cargo build --release
```

**2. Start the server (on public VPS):**
```bash
HTTP_ADDR=0.0.0.0:8080 ./target/release/tunnel-server
```

**3. Start the client (on your dev machine):**
```bash
# HTTP connection (no TLS)
SERVER_ADDR=http://<SERVER_IP>:8080 LOCAL_PORT=3000 ./target/release/tunnel-client

# HTTPS connection (with TLS)
SERVER_ADDR=https://<SERVER_DOMAIN> LOCAL_PORT=3000 ./target/release/tunnel-client
```

**4. Send HTTP requests:**
```bash
curl http://<SERVER_IP>:8080/webhook
# Request is forwarded to http://127.0.0.1:3000/webhook on your dev machine
```

### Docker Deployment

**Server-only deployment (typical use case):**
```bash
# Build server image
docker build -f Dockerfile.server -t speedforce-server .

# Run server
docker run -d \
  -p 8080:8080 \
  -e RUST_LOG=info \
  --name speedforce-server \
  speedforce-server
```

**Client-only deployment:**
```bash
# Build client image
docker build -f Dockerfile.client -t speedforce-client .

# Run client connecting to your server
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e LOCAL_PORT=3000 \
  -e RUST_LOG=info \
  --network host \
  --name speedforce-client \
  speedforce-client
```

**Full stack with docker-compose:**
```bash
docker-compose up -d
```

**Note:**
- **Server:** Typically runs in Docker on a public VPS
- **Client:** Can run natively on your dev machine OR in Docker with `--network host`
- **Network Mode:** Use `--network host` so the client can access localhost services

## Configuration

### Environment Variables

**tunnel-server:**
- `HTTP_ADDR` - Server bind address for both HTTP and tunnel connections (default: `0.0.0.0:8080`)
- `TUNNEL_AUTH` - Optional Basic Auth credentials in format `username:password` (default: none, auth disabled)
- `RUST_LOG` - Logging level (default: `info`, options: `debug`, `info`, `warn`, `error`)

**tunnel-client:**
- `SERVER_ADDR` - Server address with protocol (default: `http://127.0.0.1:8080`)
  - Supports: `https://example.com` (TLS on port 443)
  - Supports: `https://example.com:8443` (TLS on custom port)
  - Supports: `http://example.com:8080` (no TLS)
  - Supports: `example.com:8080` (no TLS, backward compat)
- `LOCAL_PORT` - Local HTTP service port (default: `3000`)
- `TUNNEL_AUTH` - Optional Basic Auth credentials in format `username:password` (default: none)
- `RUST_LOG` - Logging level (default: `info`)

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐         ┌─────────────┐
│   External  │  HTTP   │    Tunnel    │  HTTP   │    Tunnel    │  HTTP   │    Local    │
│   Client    │────────>│    Server    │ Upgrade │    Client    │────────>│   Service   │
│  (webhook)  │         │  (VPS:8080)  │  (101)  │ (dev machine)│         │ (localhost) │
└─────────────┘         └──────────────┘         └──────────────┘         └─────────────┘
                              │                          │
                              │                          │
                        Port 8080 (HTTP)           Port 3000 (HTTP)
                   (Multiplexed HTTP + Tunnel)
```

**Data Flow:**
1. Client connects to server port 8080 via HTTP
2. Client sends `GET /tunnel` with `Upgrade: tunnel` header
3. Server responds with `101 Switching Protocols`
4. Connection upgraded to raw TCP tunnel protocol
5. External HTTP request → Server wraps in TunnelRequest → Sends over upgraded connection
6. Client receives TunnelRequest → Forwards to local service (port 3000)
7. Local service responds → Client wraps in TunnelResponse
8. Client sends TunnelResponse → Server receives it
9. Server returns HTTP response → External client

## Protocol

### HTTP Upgrade Handshake

**Client → Server:**
```http
GET /tunnel HTTP/1.1
Host: example.com:8080
Upgrade: tunnel
Connection: Upgrade
```

**Server → Client:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: tunnel
Connection: Upgrade
```

After the 101 response, the connection switches to the tunnel protocol.

### Tunnel Framing Format

All messages over the upgraded connection use length-prefixed framing:

```
[4 bytes: u32 big-endian length][N bytes: JSON payload]
```

### Message Types

**TunnelRequest (Server → Client):**
```json
{
  "method": "POST",
  "path": "/webhook?source=github",
  "headers": [
    ["content-type", "application/json"],
    ["user-agent", "GitHub-Hookshot/abc123"]
  ],
  "body": "eyJldmVudCI6InB1c2gifQ=="  // base64-encoded
}
```

**TunnelResponse (Client → Server):**
```json
{
  "status": 200,
  "headers": [
    ["content-type", "application/json"]
  ],
  "body": "eyJzdWNjZXNzIjp0cnVlfQ=="  // base64-encoded
}
```

## TLS/HTTPS Support

The tunnel-client supports secure HTTPS connections with full TLS encryption and certificate validation.

### Using HTTPS

```bash
# Connect to HTTPS server (uses port 443 by default)
SERVER_ADDR=https://tunnel.example.com ./target/release/tunnel-client

# Connect to HTTPS server on custom port
SERVER_ADDR=https://tunnel.example.com:8443 ./target/release/tunnel-client

# Real example
SERVER_ADDR=https://s-server-601322859433294403.olufy-0.nortezh.com ./target/release/tunnel-client
```

### Security Features

- ✅ **Certificate Validation:** Uses Mozilla's trusted root certificates
- ✅ **SNI Support:** Proper Server Name Indication for virtual hosting
- ✅ **End-to-End Encryption:** All tunnel traffic encrypted over TLS
- ✅ **Modern TLS:** Uses rustls for secure, pure-Rust TLS implementation

### Certificate Errors

If you encounter certificate validation errors:

```bash
# Error: invalid certificate
# Solution: Ensure your server has a valid TLS certificate from a trusted CA

# Error: invalid DNS name
# Solution: SERVER_ADDR must match the certificate's Common Name or SAN
```

For self-signed certificates or testing, consider using a reverse proxy (nginx/Caddy) with a valid Let's Encrypt certificate.

## Basic Authentication

The tunnel server supports Basic Authentication to restrict which clients can connect.

### Enabling Authentication

**Server side:**
```bash
# Set credentials in format username:password
TUNNEL_AUTH=myuser:mypassword ./target/release/tunnel-server
```

**Client side:**
```bash
# Provide matching credentials
TUNNEL_AUTH=myuser:mypassword \
SERVER_ADDR=https://your-server.com \
./target/release/tunnel-client
```

### Docker with Authentication

**Server:**
```bash
docker run -d \
  -p 8080:8080 \
  -e TUNNEL_AUTH=myuser:mypassword \
  -e RUST_LOG=info \
  speedforce-server
```

**Client:**
```bash
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e TUNNEL_AUTH=myuser:mypassword \
  -e LOCAL_PORT=3000 \
  --network host \
  speedforce-client
```

### Security Notes

⚠️ **Important:** Basic Auth sends credentials in base64 encoding (NOT encryption)

**Best Practices:**
- ✅ **Always use HTTPS:** Basic Auth over HTTP exposes credentials in plaintext
- ✅ **Strong Passwords:** Use long, random passwords (e.g., generated with `openssl rand -base64 32`)
- ✅ **Environment Variables:** Never hardcode credentials in code or commit them to git
- ✅ **Rotate Regularly:** Change credentials periodically
- ⚠️ **Not a Substitute:** Use in addition to network security (VPN, firewall), not instead of it

### Backward Compatibility

Authentication is **optional** and **disabled by default**:

- If `TUNNEL_AUTH` is not set on server → No authentication required (all clients accepted)
- If `TUNNEL_AUTH` not set on client → No credentials sent
- Existing deployments continue to work without any changes

### Authentication Errors

**401 Unauthorized responses:**

```bash
# Missing credentials
ERROR: Authentication failed: Missing Authorization header

# Invalid credentials
ERROR: Authentication failed: Invalid credentials
```

The client will automatically retry with exponential backoff.

## Use Cases

✅ **Perfect for:**
- Receiving webhooks on local development machine
- Testing third-party integrations locally
- Debugging webhook payloads
- CI/CD callback endpoints during development
- Works through HTTP-only proxies/firewalls

❌ **Not recommended for:**
- Production traffic (no TLS/authentication)
- High-frequency webhooks (>100/sec)
- Large file uploads (>10MB)
- Real-time streaming

## Error Handling

| HTTP Status | Scenario | Description |
|------------|----------|-------------|
| 200-5xx | Normal | Response from local service |
| 502 | Bad Gateway | Tunnel communication failed |
| 503 | Service Unavailable | No client connected |
| 504 | Gateway Timeout | Request took longer than 30 seconds |

## Testing

Run a simple local HTTP server for testing:

```bash
# Terminal 1: Local test service
python3 -m http.server 3000

# Terminal 2: Tunnel server
cargo run --bin tunnel-server

# Terminal 3: Tunnel client
cargo run --bin tunnel-client

# Terminal 4: Send test request
curl http://localhost:8080/
```

## Troubleshooting

**Issue: Connection refused on client**
```bash
# Check server is running
netstat -an | grep 8080

# Check firewall rules
sudo ufw status

# Test direct connectivity
telnet <SERVER_IP> 8080
```

**Issue: Upgrade failed**
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin tunnel-server

# Check client logs for upgrade response
RUST_LOG=debug cargo run --bin tunnel-client
```

**Issue: Requests timing out**
```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin tunnel-server

# Check local service is running
curl http://127.0.0.1:3000/
```

**Issue: Binary data corruption**
```bash
# Verify Content-Length headers match
# Check base64 encoding/decoding
# Test with known binary file
```

## Security Considerations

### Built-in Security Features

✅ **TLS/HTTPS Support:** Client supports encrypted connections with certificate validation
✅ **Certificate Validation:** Uses Mozilla's trusted root CA certificates
✅ **End-to-End Encryption:** When using HTTPS, all tunnel traffic is encrypted
✅ **Basic Authentication:** Optional username/password protection for tunnel connections

### Security Limitations

⚠️ **Basic Auth is not encrypted:** Credentials sent in base64 (use with HTTPS!)
⚠️ **No Authorization:** All HTTP requests are forwarded to the local service
⚠️ **Server-side TLS:** The server itself doesn't handle TLS (use reverse proxy)

**Recommendations:**
- **Use HTTPS:** Always connect via `https://` in production
- **Reverse Proxy:** Deploy server behind nginx/Caddy for TLS termination
- **Firewall Rules:** Restrict server port access to trusted IPs
- **VPN/SSH Tunnel:** Additional layer for sensitive environments
- **Authentication:** Add authentication in your local service, not the tunnel
- **Development Only:** This tool is designed for dev/debugging, not production traffic

## Performance

- **Latency:** ~1-5ms overhead (serialization + framing)
- **Throughput:** 10-100 requests/second (sequential processing)
- **Memory:** ~10MB baseline per process
- **Reconnection:** Exponential backoff (1s → 2s → 4s → ... → 30s max)

## Benefits of Single Port Design

✅ **Simpler firewall configuration** - Only one port to open
✅ **Works through HTTP proxies** - Standard HTTP Upgrade mechanism
✅ **Easier deployment** - Less port management
✅ **Standard protocol** - Similar to WebSocket (RFC 7230)

## Project Structure

```
speedforce/
├── Cargo.toml                # Workspace manifest
├── tunnel-protocol/          # Shared protocol library
│   └── src/lib.rs
├── tunnel-server/            # Public HTTP endpoint
│   └── src/main.rs
├── tunnel-client/            # Dev machine client
│   └── src/main.rs
├── Dockerfile.server         # Server Docker image
├── Dockerfile.client         # Client Docker image
└── docker-compose.yml        # Full stack deployment
```

## License

MIT

## Contributing

This is a minimal MVP implementation. Feature requests and pull requests welcome!

## Acknowledgments

Built with:
- [tokio](https://tokio.rs/) - Async runtime
- [axum](https://github.com/tokio-rs/axum) - HTTP server
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [serde](https://serde.rs/) - Serialization
- [tracing](https://github.com/tokio-rs/tracing) - Structured logging
