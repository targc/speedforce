# Task: Implement Tunnel Client Listener

**Status**: pending
**Dependencies**: TK-006
**Estimated Effort**: small

## Objective

Implement the TCP listener that accepts tunnel client connections and manages the active client with "last one wins" semantics.

## Context

The server needs to listen on TUNNEL_ADDR for incoming client connections. When a new client connects, it should replace any existing client (closing the old connection) and become the active client. This task implements the tunnel listener loop that runs concurrently with the HTTP server. The listener should handle disconnections gracefully and log connection events.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Add tunnel listener function

## Detailed Steps

1. Add import:
   - `use tokio::net::TcpListener;`

2. Implement `async fn run_tunnel_listener(state: Arc<ServerState>, addr: String)`:
   - Bind TcpListener: `let listener = TcpListener::bind(&addr).await.expect("Failed to bind tunnel listener");`
   - Log: "Tunnel listener started on {addr}"
   - Enter accept loop: `loop { ... }`
   - Accept connection: `let (stream, remote_addr) = listener.accept().await?;`
   - Log: "New client connected from {remote_addr}"
   - Create TunnelConnection: `let conn = TunnelConnection::new(stream);`
   - Get write lock on active_client
   - If old connection exists, drop it and log "Replaced old client connection"
   - Store new connection: `*active = Some(conn);`
   - Release write lock (drops old connection)
   - Continue loop

3. Handle errors in accept loop:
   - Use `match listener.accept().await` instead of `?`
   - On error, log error and continue (don't crash the listener)

4. Update main function:
   - Replace TODO comment with actual spawn
   - Clone state Arc: `let listener_state = state.clone();`
   - Clone tunnel_addr: `let tunnel_addr = config.tunnel_addr.clone();`
   - Spawn listener task: `tokio::spawn(async move { run_tunnel_listener(listener_state, tunnel_addr).await });`

5. Add documentation explaining the replacement semantics

6. Test compilation with `cargo check -p tunnel-server`

## Acceptance Criteria

- [ ] TcpListener binds to configured tunnel address
- [ ] Listener accepts connections in infinite loop
- [ ] New connections create TunnelConnection instances
- [ ] Write lock is acquired to replace active client
- [ ] Old connection is closed when new one arrives (via Drop)
- [ ] Connection replacement is logged with "Replaced old client"
- [ ] Accept errors are logged but don't crash the listener
- [ ] Listener task is spawned in main function
- [ ] All connection events are logged appropriately
- [ ] `cargo check -p tunnel-server` passes without errors

## Reference

See CLAUDE.md sections:
- "Tunnel Client Management" (lines 191-220)
- "Startup Sequence" (lines 222-228)
- "Concurrent Client Replacement" (lines 477-498)
