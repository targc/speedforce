# Task: Test Client Replacement (Last One Wins)

**Status**: pending
**Dependencies**: TK-018
**Estimated Effort**: small

## Objective

Verify that when a second client connects, it replaces the first client, and subsequent requests are only forwarded to the latest connected client.

## Context

The "last client wins" semantics are important for the use case where a developer might accidentally start multiple clients or move between machines. Only one client should be active at a time, and the server should gracefully handle the replacement by closing the old connection and using the new one.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Start server:
   - Terminal 1: `HTTP_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 RUST_LOG=debug cargo run --bin tunnel-server`

2. Start two local HTTP services on different ports:
   - Terminal 2: `python3 -m http.server 3000`
   - Terminal 3: `python3 -m http.server 3001`
   - Note: Python server will show in logs which port received requests

3. Start first client pointing to port 3000:
   - Terminal 4: `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 RUST_LOG=debug cargo run --bin tunnel-client`
   - Wait for "Connected to server" message
   - Check server logs: "New client connected"

4. Test request goes to first client:
   - Terminal 5: `curl http://localhost:8080/first-client-test`
   - Check Terminal 2 (port 3000) logs: should show request received
   - Check Terminal 3 (port 3001) logs: should NOT show request

5. Start second client pointing to port 3001:
   - Terminal 6: `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3001 RUST_LOG=debug cargo run --bin tunnel-client`
   - Wait for "Connected to server" message
   - Check server logs: should show "Replaced old client connection"
   - Check Terminal 4 (first client) logs: connection should be closed/error

6. Test request now goes to second client only:
   - Terminal 5: `curl http://localhost:8080/second-client-test`
   - Check Terminal 3 (port 3001) logs: should show request received
   - Check Terminal 2 (port 3000) logs: should NOT show new request
   - Verify first client does not receive this request

7. Send multiple requests to confirm routing:
   - `curl http://localhost:8080/test1`
   - `curl http://localhost:8080/test2`
   - `curl http://localhost:8080/test3`
   - All should go to second client (port 3001) only

8. Document results and verify no split-brain behavior

## Acceptance Criteria

- [ ] Server accepts second client connection
- [ ] Server logs show "Replaced old client connection"
- [ ] First client connection is closed (detects disconnection)
- [ ] Requests after second client connects go only to second client
- [ ] No requests are sent to first client after replacement
- [ ] No split-brain behavior (requests to both clients)
- [ ] Server handles replacement without errors or crashes
- [ ] Multiple subsequent requests consistently route to latest client

## Reference

See CLAUDE.md sections:
- "Test Case 6: Multiple Clients (Last One Wins)" (lines 654-683)
- "Acceptance Criteria - Two Clients" (lines 918-922)
- "Concurrent Client Replacement" (lines 477-498)
