# Task: Add Request Timeout to Server

**Status**: pending
**Dependencies**: TK-009
**Estimated Effort**: small

## Objective

Add 30-second timeout to tunnel forwarding requests to prevent indefinite hanging if the client doesn't respond.

## Context

If the client crashes or network issues occur mid-request, the server could hang indefinitely waiting for a response. We need to wrap the tunnel communication in a timeout to fail fast and return a 502 Gateway Timeout error to the HTTP client. This follows the "fail fast" error handling philosophy.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-server/src/main.rs` - Add timeout to forward_to_tunnel

## Detailed Steps

1. Add import:
   - `use tokio::time::{timeout, Duration};`

2. Update tunnel_handler function to wrap forward_to_tunnel call:
   - Replace direct `forward_to_tunnel(&state, tunnel_req).await` call
   - Wrap in timeout: `timeout(Duration::from_secs(30), forward_to_tunnel(&state, tunnel_req)).await`
   - Handle timeout result:
     - `Ok(Ok(response))` - success, proceed normally
     - `Ok(Err(e))` - tunnel error, return 503 or 502 as before
     - `Err(_)` - timeout elapsed, log "Tunnel request timeout" and return 504 Gateway Timeout

3. Add appropriate error logging for timeout case

4. Update error response to include timeout-specific message: "Request timeout"

5. Test compilation with `cargo check -p tunnel-server`

## Acceptance Criteria

- [ ] Timeout wraps the forward_to_tunnel call
- [ ] Timeout is set to 30 seconds
- [ ] Timeout expiration returns 504 Gateway Timeout status
- [ ] Timeout error is logged with clear message
- [ ] Successful responses are unaffected by timeout wrapper
- [ ] Tunnel errors still return appropriate 502/503 codes
- [ ] `cargo check -p tunnel-server` passes without errors

## Reference

See CLAUDE.md sections:
- "HTTP Request Timeout" (lines 500-517)
- "Error Handling" (lines 182-189)
