# Task: Create Client Configuration and Main Entry Point

**Status**: pending
**Dependencies**: TK-004
**Estimated Effort**: small

## Objective

Set up tunnel-client's main.rs with environment variable configuration parsing, logging initialization, and basic entry point structure.

## Context

The tunnel-client needs to read SERVER_ADDR and LOCAL_PORT from environment variables, initialize logging, and establish the foundation for the reconnection loop. This task sets up the client's main structure following the same lean configuration approach as the server.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-client/src/main.rs` - Create client entry point

## Detailed Steps

1. Create `tunnel-client/src/main.rs` with necessary imports:
   - `use std::env;`
   - `use std::time::Duration;`
   - `use tokio::time::sleep;`
   - `use tracing::{info, debug, error};`

2. Define configuration struct:
   - `struct Config { server_addr: String, local_port: u16 }`

3. Implement `fn load_config() -> Config`:
   - Read `SERVER_ADDR` from env with default "127.0.0.1:7000"
   - Read `LOCAL_PORT` from env with default "3000", parse as u16
   - Return Config instance

4. Add constants for reconnection backoff:
   - `const INITIAL_BACKOFF: Duration = Duration::from_secs(1);`
   - `const MAX_BACKOFF: Duration = Duration::from_secs(30);`
   - `const BACKOFF_MULTIPLIER: u32 = 2;`

5. Implement main function:
   - Add `#[tokio::main]` attribute
   - Make function `async fn main()`
   - Initialize tracing subscriber: `tracing_subscriber::fmt::init();`
   - Load configuration: `let config = load_config();`
   - Log startup: "Starting client - will forward to http://127.0.0.1:{local_port}"
   - Add TODO comment for reconnection loop
   - For now, just sleep indefinitely to test compilation

6. Add documentation explaining the configuration options

7. Test compilation with `cargo check -p tunnel-client`

## Acceptance Criteria

- [ ] Config struct contains server_addr and local_port fields
- [ ] load_config reads environment variables with correct defaults
- [ ] LOCAL_PORT is parsed as u16
- [ ] Backoff constants are defined with correct values
- [ ] Tracing subscriber is initialized in main
- [ ] Main function is marked with #[tokio::main]
- [ ] Startup log message shows local forwarding URL
- [ ] TODO comment indicates where reconnection loop will go
- [ ] `cargo check -p tunnel-client` passes without errors

## Reference

See CLAUDE.md sections:
- "Configuration" (lines 240-245)
- "Startup Sequence" (lines 322-327)
- "Backoff Strategy" (lines 277-282)
