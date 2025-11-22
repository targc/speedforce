# Task: Test Basic Request Forwarding

**Status**: pending
**Dependencies**: TK-014
**Estimated Effort**: small

## Objective

Manually test that the complete system works end-to-end with basic HTTP request forwarding.

## Context

This is the first integration test to verify the core functionality. We'll start a simple local HTTP server, run the tunnel server and client, and verify that HTTP requests sent to the tunnel server are forwarded to the local service and responses come back correctly. This validates the entire request-response cycle.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Terminal 1 - Start a simple local HTTP service:
   - `cd /Users/tar/Documents/alpha/speedforce`
   - `python3 -m http.server 3000`
   - Verify it's running by curling: `curl http://localhost:3000/`

2. Terminal 2 - Start tunnel server:
   - `cd /Users/tar/Documents/alpha/speedforce`
   - `HTTP_ADDR=0.0.0.0:8080 TUNNEL_ADDR=0.0.0.0:7000 cargo run --bin tunnel-server`
   - Verify log shows: "Tunnel listener started on 0.0.0.0:7000"
   - Verify log shows: "HTTP server started on 0.0.0.0:8080"

3. Terminal 3 - Start tunnel client:
   - `cd /Users/tar/Documents/alpha/speedforce`
   - `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 cargo run --bin tunnel-client`
   - Verify log shows: "Connected to server at 127.0.0.1:7000"
   - Check server logs show: "New client connected"

4. Terminal 4 - Test basic GET request:
   - `curl -v http://localhost:8080/`
   - Expected: Response from Python HTTP server (HTML directory listing or 404)
   - Verify response is complete and correct
   - Check server logs show HTTP request received
   - Check client logs show forwarding to local service

5. Test GET request with path:
   - Create a test file: `echo "test content" > /tmp/test.txt`
   - `curl -v http://localhost:8080/../../tmp/test.txt` (Python server should resolve this)
   - Verify response contains "test content"

6. Test with query parameters:
   - `curl -v "http://localhost:8080/test?param1=value1&param2=value2"`
   - Check client logs to verify full path with query is preserved

7. Document results in this task file (add checklist items)

## Acceptance Criteria

- [ ] Python HTTP server runs on port 3000
- [ ] Tunnel server starts without errors on ports 8080 and 7000
- [ ] Tunnel client connects successfully to server
- [ ] Server logs show "New client connected"
- [ ] GET request to server is forwarded to local service
- [ ] Response from local service is returned to curl
- [ ] Path and query parameters are preserved correctly
- [ ] Server logs show method and path for each request
- [ ] Client logs show forwarding and response status
- [ ] All test requests complete successfully

## Reference

See CLAUDE.md sections:
- "Test Case 1: Basic Forwarding" (lines 539-567)
- "Acceptance Criteria - Basic Forwarding" (lines 898-904)
