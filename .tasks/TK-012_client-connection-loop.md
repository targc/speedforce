# Task: Implement Client Reconnection Loop

**Status**: pending
**Dependencies**: TK-011
**Estimated Effort**: small

## Objective

Implement the infinite reconnection loop with exponential backoff that attempts to connect to the server and handles connection failures gracefully.

## Context

The client must continuously attempt to connect to the tunnel server, retrying forever with exponential backoff on failures. When a connection succeeds, it should enter the request forwarding phase (to be implemented in next task). When the connection drops, it should return to the reconnection phase. This ensures the client is resilient and automatically recovers from network issues.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-client/src/main.rs` - Add reconnection loop

## Detailed Steps

1. Add import:
   - `use tokio::net::TcpStream;`

2. Create placeholder for connection handler:
   - `async fn handle_connection(stream: TcpStream, local_port: u16) -> Result<(), Box<dyn std::error::Error>>`
   - For now, just return Ok(()) immediately with TODO comment
   - This will be implemented in the next task

3. Replace main function's TODO with reconnection loop:
   - Initialize backoff: `let mut backoff = INITIAL_BACKOFF;`
   - Enter infinite loop: `loop { ... }`
   - Attempt connection: `match TcpStream::connect(&config.server_addr).await { ... }`
   - On Ok(stream):
     - Log: "Connected to server at {server_addr}"
     - Reset backoff: `backoff = INITIAL_BACKOFF;`
     - Call handler: `if let Err(e) = handle_connection(stream, config.local_port).await { ... }`
     - Log error if handler fails: "Connection error: {e}"
     - Log: "Disconnected from server"
   - On Err(e):
     - Log: "Connection failed: {e}"
   - Sleep with current backoff: `sleep(backoff).await;`
   - Increase backoff: `backoff = std::cmp::min(backoff * BACKOFF_MULTIPLIER, MAX_BACKOFF);`
   - Log: "Reconnecting in {backoff:?}..."

4. Add documentation explaining the reconnection strategy

5. Test compilation with `cargo check -p tunnel-client`

## Acceptance Criteria

- [ ] Infinite loop attempts to connect to server
- [ ] Successful connections log "Connected to server"
- [ ] Successful connections reset backoff to initial value
- [ ] Failed connections log error message
- [ ] Backoff increases by multiplier after each failure
- [ ] Backoff is capped at MAX_BACKOFF
- [ ] Sleep duration between retries follows backoff
- [ ] Connection handler is called when connection succeeds
- [ ] Loop continues indefinitely (never exits)
- [ ] All state transitions are logged clearly
- [ ] `cargo check -p tunnel-client` passes without errors

## Reference

See CLAUDE.md sections:
- "Connection Logic" (lines 254-282)
- "Architecture" (lines 247-252)
