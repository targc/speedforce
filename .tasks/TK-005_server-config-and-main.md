# Task: Create Server Configuration and Main Entry Point

**Status**: pending
**Dependencies**: TK-004
**Estimated Effort**: small

## Objective

Set up tunnel-server's main.rs with environment variable configuration parsing, logging initialization, and basic entry point structure.

## Context

The tunnel-server needs to read configuration from environment variables (HTTP_ADDR, TUNNEL_ADDR), initialize structured logging with tracing, and set up the async runtime. This task establishes the foundation for the server without implementing the core logic yet. Following the lean philosophy, we use simple environment variable parsing with defaults.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Create server entry point

## Detailed Steps

1. Create `tunnel-server/src/main.rs` with necessary imports:
   - `use std::env;`
   - `use std::sync::Arc;`
   - `use tokio::sync::RwLock;`
   - `use tracing::{info, debug, error};`

2. Define configuration struct:
   - `struct Config { http_addr: String, tunnel_addr: String }`

3. Implement `fn load_config() -> Config`:
   - Read `HTTP_ADDR` from env with default "0.0.0.0:8080"
   - Read `TUNNEL_ADDR` from env with default "0.0.0.0:7000"
   - Return Config instance

4. Create placeholder for shared state (to be implemented in next tasks):
   - Define empty `struct ServerState` (will add fields later)
   - Add `impl ServerState { fn new() -> Self }` constructor

5. Implement main function:
   - Add `#[tokio::main]` attribute
   - Make function `async fn main()`
   - Initialize tracing subscriber: `tracing_subscriber::fmt::init();`
   - Load configuration: `let config = load_config();`
   - Initialize shared state: `let state = Arc::new(ServerState::new());`
   - Log startup message: "Server starting - HTTP on {http_addr}, Tunnel on {tunnel_addr}"
   - Add TODO comments for spawning tunnel listener and starting HTTP server
   - Add `tokio::signal::ctrl_c().await` to keep process running
   - Log shutdown message

6. Test compilation with `cargo check -p tunnel-server`

## Acceptance Criteria

- [ ] Config struct contains http_addr and tunnel_addr fields
- [ ] load_config reads environment variables with correct defaults
- [ ] Tracing subscriber is initialized in main
- [ ] Main function is marked with #[tokio::main]
- [ ] ServerState placeholder is defined
- [ ] Startup log message includes both addresses
- [ ] TODO comments indicate where core logic will be added
- [ ] `cargo check -p tunnel-server` passes without errors
- [ ] Server compiles and can run (even if it just waits for Ctrl+C)

## Reference

See CLAUDE.md sections:
- "Configuration" (lines 140-145)
- "Startup Sequence" (lines 222-228)
- "Logging" (lines 1120-1146)
