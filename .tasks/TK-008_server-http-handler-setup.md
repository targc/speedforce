# Task: Set Up HTTP Server with Axum

**Status**: pending
**Dependencies**: TK-007
**Estimated Effort**: small

## Objective

Set up the axum HTTP server with a catch-all route handler and shared state injection.

## Context

The HTTP server needs to accept requests on all methods and paths, then forward them through the tunnel. We'll use axum's routing system with a catch-all handler and share the ServerState via axum's State extractor. This task sets up the HTTP server structure; the actual forwarding logic will be implemented in the next task.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Add HTTP server setup

## Detailed Steps

1. Add imports:
   - `use axum::{Router, routing::any, extract::State, http::{Request, StatusCode}, body::Body, response::{Response, IntoResponse}};`

2. Create placeholder handler function:
   - `async fn tunnel_handler(State(state): State<Arc<ServerState>>, req: Request<Body>) -> Result<Response, StatusCode>`
   - For now, return a simple response: `Ok(Response::new(Body::from("TODO: implement forwarding")))`
   - Add TODO comment indicating this will be replaced with actual forwarding logic

3. Implement `async fn run_http_server(state: Arc<ServerState>, addr: String)`:
   - Create axum app: `let app = Router::new().fallback(tunnel_handler).with_state(state);`
   - Parse address: `let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind HTTP server");`
   - Log: "HTTP server started on {addr}"
   - Start server: `axum::serve(listener, app).await.expect("HTTP server error");`

4. Update main function:
   - Replace HTTP server TODO with actual spawn
   - Clone state: `let http_state = state.clone();`
   - Clone http_addr: `let http_addr = config.http_addr.clone();`
   - Spawn HTTP server: `tokio::spawn(async move { run_http_server(http_state, http_addr).await });`

5. Remove the `ctrl_c` wait from main since the HTTP server will block

6. Add documentation for the HTTP server setup

7. Test compilation and basic functionality:
   - `cargo check -p tunnel-server`
   - Can test running server and curling to verify it responds with placeholder

## Acceptance Criteria

- [ ] Axum Router is created with fallback route
- [ ] ServerState is injected via with_state
- [ ] tunnel_handler function accepts State and Request parameters
- [ ] Handler returns placeholder response for now
- [ ] HTTP server binds to configured address
- [ ] Server startup is logged
- [ ] HTTP server task is spawned in main
- [ ] `cargo check -p tunnel-server` passes without errors
- [ ] Server can start and respond to curl requests with placeholder

## Reference

See CLAUDE.md sections:
- "HTTP Request Handling" (lines 161-189)
- "Startup Sequence" (lines 222-228)
- Dependencies for axum (lines 362-378)
