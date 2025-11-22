# Task: Implement Server Shared State Structure

**Status**: pending
**Dependencies**: TK-005
**Estimated Effort**: small

## Objective

Implement the ServerState struct with thread-safe storage for the active tunnel client connection.

## Context

The server needs to maintain a single active TCP tunnel connection that can be accessed by multiple HTTP handler tasks (read access) and replaced by the tunnel listener task (write access). We use Arc<RwLock<Option<TunnelConnection>>> to enable safe concurrent access with the "last client wins" semantics. The TunnelConnection struct wraps split TCP read/write halves.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Update ServerState definition

## Detailed Steps

1. Add imports to `main.rs`:
   - `use tokio::net::TcpStream;`
   - `use tokio::io::{BufReader, ReadHalf, WriteHalf};`

2. Define `TunnelConnection` struct:
   - Field `reader: BufReader<ReadHalf<TcpStream>>`
   - Field `writer: WriteHalf<TcpStream>`
   - Both fields should be public (for now, can refactor to private with methods later)

3. Implement `TunnelConnection` methods:
   - `pub fn new(stream: TcpStream) -> Self`
   - Split stream into halves: `let (read_half, write_half) = tokio::io::split(stream);`
   - Wrap read half in BufReader: `let reader = BufReader::new(read_half);`
   - Return TunnelConnection with reader and writer

4. Update `ServerState` struct:
   - Replace empty struct with: `struct ServerState { active_client: RwLock<Option<TunnelConnection>> }`

5. Update `ServerState::new()` implementation:
   - Return `Self { active_client: RwLock::new(None) }`

6. Add helper methods to ServerState:
   - `pub async fn set_client(&self, conn: TunnelConnection)` - acquires write lock and sets active_client to Some(conn)
   - `pub async fn has_client(&self) -> bool` - acquires read lock and returns whether active_client is Some
   - Note: We'll add the actual request forwarding method in a later task

7. Add documentation comments explaining the concurrent access pattern

8. Test compilation with `cargo check -p tunnel-server`

## Acceptance Criteria

- [ ] TunnelConnection struct wraps BufReader<ReadHalf> and WriteHalf
- [ ] TunnelConnection::new properly splits TcpStream and wraps reader
- [ ] ServerState contains RwLock<Option<TunnelConnection>>
- [ ] ServerState::new initializes with None
- [ ] set_client method acquires write lock and stores connection
- [ ] has_client method acquires read lock and checks for active client
- [ ] All structs and methods are well-documented
- [ ] `cargo check -p tunnel-server` passes without errors

## Reference

See CLAUDE.md sections:
- "Architecture" (lines 147-159)
- "Connection Storage" (lines 209-216)
- "Concurrency" (lines 218-220)
- "Concurrent Client Replacement" (lines 477-498)
