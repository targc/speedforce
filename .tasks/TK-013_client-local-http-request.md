# Task: Implement Local HTTP Request Forwarding

**Status**: pending
**Dependencies**: TK-012
**Estimated Effort**: small

## Objective

Implement the function that converts a TunnelRequest into an HTTP request to the local service and handles the response.

## Context

When the client receives a TunnelRequest from the server, it needs to make an actual HTTP request to the local service (localhost:LOCAL_PORT). This involves decoding the base64 body, constructing an HTTP request with the correct method/path/headers, executing it with reqwest, and handling both success and failure cases. On failure, we return a 502 error response rather than crashing.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-client/src/main.rs` - Add local HTTP forwarding function

## Detailed Steps

1. Add import:
   - `use reqwest;`

2. Implement `async fn forward_to_local(tunnel_req: tunnel_protocol::TunnelRequest, local_port: u16) -> tunnel_protocol::TunnelResponse`:
   - Construct URL: `let url = format!("http://127.0.0.1:{}{}", local_port, tunnel_req.path);`
   - Log debug: "Forwarding to local service url={url}"
   - Decode body: `let body_bytes = tunnel_protocol::decode_body(&tunnel_req.body).unwrap_or_else(|e| { error!("Body decode error: {e}"); vec![] });`
   - Create reqwest client: `let client = reqwest::Client::new();`
   - Build request:
     - Start with method: `let mut req = client.request(tunnel_req.method.parse().unwrap_or(reqwest::Method::GET), &url);`
     - Add headers: iterate tunnel_req.headers and call `req = req.header(name, value);`
     - Add body: `req = req.body(body_bytes);`
   - Execute request: `match req.send().await { ... }`
   - On Ok(response):
     - Extract status: `response.status().as_u16()`
     - Extract headers: `response.headers().iter().map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string())).collect()`
     - Extract body: `response.bytes().await.unwrap_or_default()`
     - Encode body: `tunnel_protocol::encode_body(&resp_body)`
     - Log debug: "Local service responded status={status}"
     - Return TunnelResponse { status, headers, body: encoded_body }
   - On Err(e):
     - Log error: "Local HTTP request failed: {e}"
     - Create error response:
       - status: 502
       - headers: vec![("content-type".to_string(), "text/plain".to_string())]
       - body: encode_body("Local service unavailable".as_bytes())
     - Return error response

3. Add documentation explaining error handling strategy

4. Test compilation with `cargo check -p tunnel-client`

## Acceptance Criteria

- [ ] URL is constructed with local_port and path from tunnel request
- [ ] Request body is base64-decoded before forwarding
- [ ] HTTP method is parsed from string (defaults to GET on parse error)
- [ ] All headers from tunnel request are added to local request
- [ ] Response status, headers, and body are extracted correctly
- [ ] Response body is base64-encoded before returning
- [ ] Local HTTP failures return 502 with clear error message
- [ ] All steps are logged at appropriate levels (debug for normal flow, error for failures)
- [ ] `cargo check -p tunnel-client` passes without errors

## Reference

See CLAUDE.md sections:
- "Request Forwarding Logic" (lines 284-320)
- "Error Handling" (lines 306-320)
