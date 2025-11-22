# Task: Implement Client Tunnel Request Processing Loop

**Status**: pending
**Dependencies**: TK-013
**Estimated Effort**: small

## Objective

Implement the main request processing loop that receives TunnelRequests from the server, forwards them to the local service, and sends TunnelResponses back.

## Context

Once connected to the server, the client enters a tight loop: read TunnelRequest frame, forward to local HTTP service, send TunnelResponse frame back. This loop continues until the TCP connection breaks (EOF or error), at which point we exit and return to the reconnection loop. The loop uses the protocol helper functions for framing and serialization.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-client/src/main.rs` - Implement handle_connection

## Detailed Steps

1. Add imports:
   - `use tokio::io::{BufReader, split};`
   - `use tunnel_protocol::{recv_request, send_response};`

2. Replace handle_connection placeholder implementation:
   - Split stream: `let (read_half, write_half) = split(stream);`
   - Wrap reader in BufReader: `let mut reader = BufReader::new(read_half);`
   - Make writer mutable: `let mut writer = write_half;`
   - Enter processing loop: `loop { ... }`
   - Receive request: `let tunnel_req = recv_request(&mut reader).await?;`
   - Log debug: "Received tunnel request method={} path={}", tunnel_req.method, tunnel_req.path
   - Forward to local: `let tunnel_resp = forward_to_local(tunnel_req, local_port).await;`
   - Send response: `send_response(&mut writer, &tunnel_resp).await?;`
   - Loop continues until error (EOF or network issue)

3. Handle loop exit:
   - When recv_request or send_response returns error, propagate it up
   - The error will be caught in main's reconnection loop
   - Log will show "Connection error" or "Disconnected from server"

4. Add documentation explaining the sequential processing model

5. Test compilation with `cargo check -p tunnel-client`

## Acceptance Criteria

- [ ] TcpStream is split into read and write halves
- [ ] Read half is wrapped in BufReader
- [ ] Loop receives requests using recv_request
- [ ] Each request is forwarded to local service
- [ ] Responses are sent using send_response
- [ ] Loop continues until connection breaks
- [ ] Connection errors propagate up to reconnection loop
- [ ] Request method and path are logged for each received request
- [ ] Sequential processing (one request at a time) is maintained
- [ ] `cargo check -p tunnel-client` passes without errors

## Reference

See CLAUDE.md sections:
- "Request Forwarding Logic" (lines 284-320)
- "Protocol Constraints" (lines 123-128)
