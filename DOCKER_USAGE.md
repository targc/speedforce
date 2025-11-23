# Running Speedforce Client in Docker

This guide shows how to run the tunnel-client in a Docker container to connect to your remote speedforce server.

## Why Use Docker for the Client?

- ✅ **Consistent environment** across different machines
- ✅ **Easy deployment** on cloud VMs or containers
- ✅ **Simplified dependencies** - no need to install Rust
- ✅ **Portable** - run anywhere Docker is available

## Quick Start

### 1. Build the Client Image

```bash
cd /path/to/speedforce
docker build -f Dockerfile.client -t speedforce-client .
```

### 2. Run the Client

**Connect to HTTPS server:**
```bash
docker run -d \
  -e SERVER_ADDR=https://s-server-601322859433294403.olufy-0.nortezh.com \
  -e LOCAL_PORT=3000 \
  -e RUST_LOG=info \
  --network host \
  --name speedforce-client \
  speedforce-client
```

**Connect to HTTP server:**
```bash
docker run -d \
  -e SERVER_ADDR=http://example.com:8080 \
  -e LOCAL_PORT=3000 \
  -e RUST_LOG=info \
  --network host \
  --name speedforce-client \
  speedforce-client
```

### 3. View Logs

```bash
docker logs -f speedforce-client
```

Expected output:
```
[INFO] Starting client - will connect to s-server-601322859433294403.olufy-0.nortezh.com:443 (TLS: true)
[INFO] TCP connection established
[INFO] TLS connection established
[INFO] HTTP Upgrade successful
[INFO] Connected and upgraded to tunnel protocol
```

### 4. Stop the Client

```bash
docker stop speedforce-client
docker rm speedforce-client
```

## Network Modes

### Using `--network host` (Recommended)

This allows the Docker container to access services running on your host machine's localhost.

```bash
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e LOCAL_PORT=3000 \
  --network host \
  --name speedforce-client \
  speedforce-client
```

**Pros:**
- ✅ Simple configuration
- ✅ Can forward to localhost:3000 on host machine
- ✅ No port mapping needed

**Cons:**
- ⚠️ Only works on Linux (macOS/Windows have limitations)
- ⚠️ Container shares host network stack

### Using Bridge Network + Port Mapping (macOS/Windows)

On macOS and Windows, `--network host` doesn't work the same way. Instead, you need to forward to a service in another container or use `host.docker.internal`:

```bash
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e LOCAL_PORT=3000 \
  -e RUST_LOG=info \
  --name speedforce-client \
  speedforce-client
```

Then, in your docker-compose.yml, run your local service:
```yaml
version: '3.8'
services:
  local-app:
    image: your-app-image
    ports:
      - "3000:3000"
    networks:
      - speedforce

  tunnel-client:
    image: speedforce-client
    environment:
      SERVER_ADDR: https://your-server.com
      LOCAL_PORT: 3000
    networks:
      - speedforce
    depends_on:
      - local-app

networks:
  speedforce:
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `SERVER_ADDR` | Remote server URL | `https://tunnel.example.com` |
| `LOCAL_PORT` | Local service port to forward to | `3000` |
| `RUST_LOG` | Log level | `info`, `debug`, `warn`, `error` |

## Common Use Cases

### 1. Forward to Local Development Server

```bash
# Terminal 1: Start your dev server
npm run dev  # Runs on localhost:3000

# Terminal 2: Start tunnel client in Docker
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e LOCAL_PORT=3000 \
  --network host \
  --name speedforce-client \
  speedforce-client
```

### 2. Forward to Dockerized Service

```bash
# Create a docker network
docker network create speedforce

# Run your app
docker run -d \
  --name my-app \
  --network speedforce \
  -p 3000:3000 \
  my-app-image

# Run tunnel client pointing to the containerized app
docker run -d \
  -e SERVER_ADDR=https://your-server.com \
  -e LOCAL_PORT=3000 \
  --network speedforce \
  --name speedforce-client \
  speedforce-client
```

### 3. Multiple Clients (Different Services)

```bash
# Client 1: Forward to service on port 3000
docker run -d \
  -e SERVER_ADDR=https://server1.example.com \
  -e LOCAL_PORT=3000 \
  --network host \
  --name speedforce-client-1 \
  speedforce-client

# Client 2: Forward to service on port 4000
docker run -d \
  -e SERVER_ADDR=https://server2.example.com \
  -e LOCAL_PORT=4000 \
  --network host \
  --name speedforce-client-2 \
  speedforce-client
```

## Troubleshooting

### Issue: "Connection refused" to localhost

**Problem:** Container can't access localhost:3000

**Solution:** Use `--network host` on Linux, or use `host.docker.internal:3000` instead of `localhost:3000` on macOS/Windows.

### Issue: TLS certificate validation failed

**Problem:** Server certificate is invalid or self-signed

**Solution:** Ensure your server has a valid certificate from a trusted CA (like Let's Encrypt).

### Issue: Client keeps disconnecting

**Problem:** Network issues or server restarting

**Solution:** The client has auto-reconnect with exponential backoff. Check logs:
```bash
docker logs speedforce-client
```

### Issue: Can't see logs

**Solution:**
```bash
# View logs
docker logs -f speedforce-client

# Enable debug logging
docker run -e RUST_LOG=debug ...
```

## Docker Compose Example

Full setup with client and local service:

```yaml
version: '3.8'

services:
  my-app:
    image: my-app:latest
    ports:
      - "3000:3000"
    networks:
      - speedforce

  tunnel-client:
    image: speedforce-client:latest
    environment:
      SERVER_ADDR: https://s-server-601322859433294403.olufy-0.nortezh.com
      LOCAL_PORT: 3000
      RUST_LOG: info
    networks:
      - speedforce
    depends_on:
      - my-app
    restart: unless-stopped

networks:
  speedforce:
    driver: bridge
```

Run with:
```bash
docker-compose up -d
docker-compose logs -f tunnel-client
```

## Performance Considerations

- **Image Size:** ~115MB (includes Rust binary + CA certificates)
- **Memory:** ~10-20MB per client
- **CPU:** Minimal (mostly I/O bound)
- **Startup Time:** <1 second

## Security Notes

- ✅ TLS certificates are validated using Mozilla's root CA bundle
- ✅ All tunnel traffic is encrypted when using HTTPS
- ⚠️ Container runs as root by default (consider adding USER directive for production)
- ⚠️ No authentication between client and server (add firewall rules)
