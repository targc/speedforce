# Task: Implement HTTP Request to Tunnel Forwarding

**Status**: pending
**Dependencies**: TK-008
**Estimated Effort**: medium

## Objective

Implement the complete HTTP request forwarding logic: extract HTTP request details, convert to TunnelRequest, send through tunnel, receive TunnelResponse, and convert back to HTTP response.

## Context

This is the core server functionality. When an HTTP request arrives, we need to: (1) check if a client is connected, (2) extract method, path with query, headers, and body, (3) base64-encode the body, (4) serialize to TunnelRequest and send over TCP, (5) wait for TunnelResponse, (6) decode and convert back to HTTP response. Error cases include no client connected (503) and tunnel communication failures (502).

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Replace tunnel_handler placeholder

## Detailed Steps

1. Add imports:
   - `use axum::body::to_bytes;`
   - `use tunnel_protocol::{TunnelRequest, TunnelResponse, encode_body, decode_body, send_request, recv_response};`

2. Implement helper to extract request details:
   - Create `async fn extract_http_request(req: Request<Body>) -> Result<TunnelRequest, String>`
   - Extract method: `req.method().as_str().to_string()`
   - Extract path with query: `req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or("/").to_string()`
   - Extract headers as Vec<(String, String)>: iterate headers, convert to (name.to_string(), value.to_str().unwrap_or("").to_string())
   - Extract body bytes: `to_bytes(req.into_body(), usize::MAX).await.map_err(|e| e.to_string())?`
   - Encode body: `let encoded_body = encode_body(&body_bytes);`
   - Return Ok(TunnelRequest { method, path, headers, body: encoded_body })

3. Implement helper to forward through tunnel:
   - Create `async fn forward_to_tunnel(state: &ServerState, tunnel_req: TunnelRequest) -> Result<TunnelResponse, String>`
   - Acquire read lock: `let mut client_guard = state.active_client.write().await;`
   - Check if client exists: if None, return Err("No tunnel client connected")
   - Get mutable references to reader and writer from the TunnelConnection
   - Send request: `send_request(&mut client.writer, &tunnel_req).await.map_err(|e| format!("Tunnel write failed: {}", e))?`
   - Receive response: `recv_response(&mut client.reader).await.map_err(|e| format!("Tunnel read failed: {}", e))?`
   - Return Ok(tunnel_response)

4. Implement helper to convert TunnelResponse to HTTP Response:
   - Create `fn build_http_response(tunnel_resp: TunnelResponse) -> Result<Response, String>`
   - Decode body: `let body_bytes = decode_body(&tunnel_resp.body).map_err(|e| format!("Invalid body encoding: {}", e))?`
   - Create response builder with status: `let mut response = Response::builder().status(tunnel_resp.status);`
   - Add headers: iterate tunnel_resp.headers and insert each one
   - Set body: `response.body(Body::from(body_bytes)).map_err(|e| e.to_string())`

5. Replace tunnel_handler implementation:
   - Extract request: `let tunnel_req = extract_http_request(req).await.map_err(|e| { error!("Failed to extract request: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;`
   - Log debug: "HTTP request received method={} path={}", tunnel_req.method, tunnel_req.path
   - Forward through tunnel: `let tunnel_resp = forward_to_tunnel(&state, tunnel_req).await`
   - On error: check error message and return appropriate status (503 for "No tunnel client", 502 for others)
   - Build HTTP response: `build_http_response(tunnel_resp).map_err(|e| { error!("Failed to build response: {}", e); StatusCode::BAD_GATEWAY })?`
   - Return Ok(response)

6. Add error logging for each failure case

7. Test compilation with `cargo check -p tunnel-server`

## Acceptance Criteria

- [ ] Method, path with query, headers, and body are correctly extracted from HTTP request
- [ ] Request body is base64-encoded before sending
- [ ] TunnelRequest is serialized and sent via send_request
- [ ] TunnelResponse is received via recv_response
- [ ] Response body is base64-decoded after receiving
- [ ] HTTP response is built with correct status, headers, and body
- [ ] Returns 503 when no client is connected
- [ ] Returns 502 on tunnel communication errors
- [ ] All errors are logged with appropriate detail
- [ ] Debug logs show method and path for each request
- [ ] `cargo check -p tunnel-server` passes without errors

## Reference

See CLAUDE.md sections:
- "HTTP Request Handling" (lines 161-189)
- "Path Preservation" (lines 463-475)
- "Header Conversion" (lines 446-461)
