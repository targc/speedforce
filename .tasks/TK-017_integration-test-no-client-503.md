# Task: Test 503 Response When Client Not Connected

**Status**: pending
**Dependencies**: TK-015
**Estimated Effort**: small

## Objective

Verify that the server returns HTTP 503 Service Unavailable with a clear error message when no tunnel client is connected.

## Context

This is an important error case - users need to understand when the tunnel client is not connected. The server should remain operational and return a helpful error rather than crashing or hanging. This validates the error handling for the "no client" scenario.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Terminal 1 - Start only the tunnel server (no client):
   - Stop any running client from previous tests (Ctrl+C)
   - Ensure server is running: `HTTP_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 cargo run --bin tunnel-server`
   - Verify server started successfully

2. Terminal 2 - Test HTTP request with no client:
   - `curl -v http://localhost:8080/test`
   - Verify response status is 503 Service Unavailable
   - Verify response body contains "No tunnel client connected"
   - Verify request completes immediately (no hanging)

3. Test multiple requests with no client:
   - `curl -v http://localhost:8080/test1`
   - `curl -v http://localhost:8080/test2`
   - Verify both return 503
   - Verify server continues accepting requests

4. Check server logs:
   - Should show HTTP requests being received
   - Should show appropriate error handling (no crashes or panics)

5. Now start the client and verify recovery:
   - Terminal 3: `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 cargo run --bin tunnel-client`
   - Wait for connection to establish
   - Terminal 2: `curl -v http://localhost:8080/test`
   - Verify request now succeeds (or returns response from local service)

6. Document results in this task file

## Acceptance Criteria

- [ ] Server returns 503 when no client is connected
- [ ] Response body contains clear error message
- [ ] Requests complete immediately without hanging
- [ ] Multiple requests all return 503 consistently
- [ ] Server continues running and accepting requests
- [ ] No crashes or panics in server logs
- [ ] Once client connects, requests succeed normally
- [ ] Server did not need to be restarted for client connection

## Reference

See CLAUDE.md sections:
- "Test Case 4: Client Not Connected (503)" (lines 597-617)
- "Acceptance Criteria - Client Down" (lines 909-912)
- "Error Handling" (lines 182-189)
