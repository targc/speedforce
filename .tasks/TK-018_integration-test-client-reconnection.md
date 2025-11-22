# Task: Test Client Automatic Reconnection

**Status**: pending
**Dependencies**: TK-017
**Estimated Effort**: small

## Objective

Verify that the tunnel client automatically reconnects when the connection is lost, with exponential backoff, and that requests work again after reconnection.

## Context

The client must be resilient to network issues and server restarts. When the connection drops, it should enter the reconnection loop with exponential backoff, and once reconnected, requests should flow normally again. This validates the auto-recovery mechanism and backoff strategy.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Start complete test environment:
   - Terminal 1: `python3 -m http.server 3000`
   - Terminal 2: Tunnel server with `RUST_LOG=debug` for detailed logs
   - Terminal 3: Tunnel client with `RUST_LOG=debug`
   - Verify all connected

2. Test successful request before disconnection:
   - Terminal 4: `curl http://localhost:8080/test1`
   - Verify success (response from Python server)

3. Simulate client crash/disconnect:
   - Terminal 3: Press Ctrl+C to kill the client
   - Terminal 4: Wait 2 seconds, then `curl http://localhost:8080/test2`
   - Verify returns 503 (no client connected)
   - Check server logs: should show client disconnection

4. Restart client and observe reconnection:
   - Terminal 3: Restart client `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 RUST_LOG=debug cargo run --bin tunnel-client`
   - Watch client logs for connection attempts
   - Should see: "Connected to server at 127.0.0.1:7000"
   - Check server logs: should show "New client connected"

5. Test requests work after reconnection:
   - Terminal 4: `curl http://localhost:8080/test3`
   - Verify success (response from Python server)
   - No server restart was needed

6. Test exponential backoff with unreachable server:
   - Stop server (Terminal 2: Ctrl+C)
   - Keep client running (Terminal 3)
   - Watch client logs for reconnection attempts
   - Verify backoff intervals: should see delays increasing (1s, 2s, 4s, 8s, 16s, 30s max)
   - Verify "Reconnecting in ..." messages show increasing durations

7. Restart server and verify client reconnects:
   - Terminal 2: Restart server
   - Watch client logs: should eventually reconnect
   - Test request works after reconnection

8. Document results and observed backoff behavior

## Acceptance Criteria

- [ ] Client detects when connection is lost
- [ ] Client enters reconnection loop automatically
- [ ] Requests return 503 while client is disconnected
- [ ] Client successfully reconnects when server is available
- [ ] Requests work normally after reconnection
- [ ] No server restart is needed for client reconnection
- [ ] Exponential backoff is observed in logs (1s → 2s → 4s → ...)
- [ ] Backoff caps at 30 seconds maximum
- [ ] Client continues retrying indefinitely
- [ ] Connection state changes are clearly logged

## Reference

See CLAUDE.md sections:
- "Test Case 5: Client Reconnection" (lines 619-652)
- "Acceptance Criteria - Client Reconnect" (lines 913-917)
- "Backoff Strategy" (lines 277-282)
