# Task: Test Server Restart Recovery

**Status**: pending
**Dependencies**: TK-020
**Estimated Effort**: small

## Objective

Verify that the client survives server restarts and automatically reconnects when the server comes back online.

## Context

In real-world scenarios, the server might be restarted for updates or crash and restart. The client should detect the disconnection, enter the reconnection loop, and successfully reconnect once the server is available again. This validates the client's resilience to server-side issues.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Start complete environment:
   - Terminal 1: `python3 -m http.server 3000`
   - Terminal 2: `RUST_LOG=debug cargo run --bin tunnel-server`
   - Terminal 3: `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 RUST_LOG=debug cargo run --bin tunnel-client`
   - Verify all connected

2. Test successful request before server restart:
   - Terminal 4: `curl http://localhost:8080/before-restart`
   - Verify success

3. Restart the server while client is running:
   - Terminal 2: Press Ctrl+C to stop server
   - Watch Terminal 3 (client): should detect disconnection
   - Client logs should show connection error and start reconnection attempts
   - Wait and observe reconnection attempts with exponential backoff

4. Let client retry for a bit (observe backoff):
   - Client should keep trying to reconnect
   - Should see logs like "Connection failed" and "Reconnecting in ..."
   - Backoff should increase: 1s, 2s, 4s, etc.

5. Restart server:
   - Terminal 2: `RUST_LOG=debug cargo run --bin tunnel-server`
   - Wait for server to start and listen

6. Observe client reconnection:
   - Client should detect server is back
   - Should see "Connected to server at 127.0.0.1:7000"
   - Server should show "New client connected"

7. Test requests work after server restart:
   - Terminal 4: `curl http://localhost:8080/after-restart`
   - Verify request succeeds
   - Check that response comes from local service

8. Test multiple restart cycles:
   - Repeat server restart (Ctrl+C and restart)
   - Verify client reconnects each time
   - Verify requests work after each reconnection

9. Document results including observed backoff intervals

## Acceptance Criteria

- [ ] Client detects server disconnection
- [ ] Client enters reconnection loop automatically
- [ ] Client continues retrying with exponential backoff
- [ ] Client successfully reconnects when server restarts
- [ ] Requests work normally after server restart
- [ ] Client did NOT need to be restarted
- [ ] Reconnection works across multiple server restart cycles
- [ ] Backoff behavior is observed (increasing delays)
- [ ] No client crashes or panics during server downtime

## Reference

See CLAUDE.md sections:
- "Test Case 8: Server Restart Recovery" (lines 707-728)
- "Acceptance Criteria - Server Restart Recovery" (lines 923-927)
