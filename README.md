# speedforce - HTTP-over-TCP Tunnel

A minimal HTTP tunnel for local development and webhook debugging. Forward HTTP requests from a public server to your local development machine via a persistent TCP connection.

## Features

- 🚀 **Simple Setup** - Two binaries, no complex configuration
- 🔄 **Auto-Reconnect** - Client automatically reconnects with exponential backoff
- 🎯 **Path Preservation** - Full URL paths and query strings preserved exactly
- 📦 **Binary Support** - Handles arbitrary binary HTTP bodies via base64 encoding
- 🔌 **Single Connection** - One active client at a time (last connected wins)
- 🐳 **Docker Ready** - Dockerfiles and docker-compose included

## Quick Start

### Binary Installation

**1. Build from source:**
```bash
cargo build --release
```

**2. Start the server (on public VPS):**
```bash
HTTP_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 ./target/release/tunnel-server
```

**3. Start the client (on your dev machine):**
```bash
SERVER_ADDR=<SERVER_IP>:7000 LOCAL_PORT=3000 ./target/release/tunnel-client
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
  -p 7000:7000 \
  -e RUST_LOG=info \
  --name speedforce-server \
  speedforce-server
```

**Full stack with docker-compose:**
```bash
docker-compose up -d
```

**Note:** Typically you run the server in Docker on a public VPS, and the client runs natively on your dev machine to forward to localhost services.

## Configuration

### Environment Variables

**tunnel-server:**
- `HTTP_ADDR` - HTTP server bind address (default: `0.0.0.0:8080`)
- `TUNNEL_ADDR` - TCP tunnel listener address (default: `0.0.0.0:7000`)
- `RUST_LOG` - Logging level (default: `info`, options: `debug`, `info`, `warn`, `error`)

**tunnel-client:**
- `SERVER_ADDR` - Server TCP address to connect (default: `127.0.0.1:7000`)
- `LOCAL_PORT` - Local HTTP service port (default: `3000`)
- `RUST_LOG` - Logging level (default: `info`)

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐         ┌─────────────┐
│   External  │  HTTP   │    Tunnel    │   TCP   │    Tunnel    │  HTTP   │    Local    │
│   Client    │────────>│    Server    │<───────>│    Client    │────────>│   Service   │
│  (webhook)  │         │  (VPS:8080)  │  Tunnel │ (dev machine)│         │ (localhost) │
└─────────────┘         └──────────────┘         └──────────────┘         └─────────────┘
                              │                          │
                              │                          │
                        Port 7000 (TCP)           Port 3000 (HTTP)
```

**Data Flow:**
1. External HTTP request → Server HTTP endpoint (port 8080)
2. Server wraps request in TunnelRequest → Sends over TCP connection (port 7000)
3. Client receives TunnelRequest → Forwards to local service (port 3000)
4. Local service responds → Client wraps in TunnelResponse
5. Client sends TunnelResponse → Server receives it
6. Server returns HTTP response → External client

## Protocol

### Framing Format

All messages over TCP use length-prefixed framing:

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

## Use Cases

✅ **Perfect for:**
- Receiving webhooks on local development machine
- Testing third-party integrations locally
- Debugging webhook payloads
- CI/CD callback endpoints during development

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
netstat -an | grep 7000

# Check firewall rules
sudo ufw status

# Test direct connectivity
telnet <SERVER_IP> 7000
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

⚠️ **This is a development tool with NO built-in security:**

- **No TLS/encryption** - All traffic is plaintext
- **No authentication** - Anyone can connect as a client
- **No authorization** - All HTTP requests are forwarded

**Recommendations:**
- Use SSH tunnel or VPN for secure communication
- Firewall the TCP port (7000) to trusted IPs only
- Never expose to public internet without additional security layers
- Consider this for development/debugging only

## Performance

- **Latency:** ~1-5ms overhead (serialization + framing)
- **Throughput:** 10-100 requests/second (sequential processing)
- **Memory:** ~10MB baseline per process
- **Reconnection:** Exponential backoff (1s → 2s → 4s → ... → 30s max)

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
