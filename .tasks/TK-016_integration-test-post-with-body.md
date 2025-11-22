# Task: Test POST Requests with JSON Body

**Status**: pending
**Dependencies**: TK-015
**Estimated Effort**: small

## Objective

Test that POST requests with JSON bodies are forwarded correctly, including Content-Type headers and body data preservation.

## Context

Many webhooks use POST with JSON payloads, so this is a critical use case. We need to verify that the body is correctly base64-encoded during transmission and decoded when forwarding to the local service, and that headers like Content-Type are preserved through the tunnel.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Ensure test environment is running (from TK-015):
   - Terminal 1: Python HTTP server on port 3000
   - Terminal 2: Tunnel server
   - Terminal 3: Tunnel client
   - All should show connected state

2. Terminal 4 - Test POST with JSON body:
   - `curl -v -X POST -H "Content-Type: application/json" -d '{"event":"push","repo":"test"}' http://localhost:8080/webhook`
   - Verify request completes
   - Check server logs: should show "method=POST path=/webhook"
   - Check client logs: should show forwarding to local service
   - Python server will return 404 (expected - endpoint doesn't exist), but verify body was received

3. Test POST with larger JSON payload:
   - Create test file with larger JSON: `echo '{"large":"data","items":["a","b","c"],"nested":{"key":"value"}}' > /tmp/payload.json`
   - `curl -v -X POST -H "Content-Type: application/json" -d @/tmp/payload.json http://localhost:8080/api/data`
   - Verify request completes without errors

4. Test POST with empty body:
   - `curl -v -X POST -H "Content-Type: application/json" http://localhost:8080/empty`
   - Verify request completes (empty body should be encoded as empty string)

5. Test POST with special characters in JSON:
   - `curl -v -X POST -H "Content-Type: application/json" -d '{"message":"Hello \"World\"","unicode":"日本語"}' http://localhost:8080/test`
   - Verify special characters and Unicode are preserved

6. Optional: Set up a simple echo server to verify body is actually forwarded:
   - Create simple Node.js or Python script that echoes POST body
   - Forward through tunnel and verify echoed body matches sent body

7. Document results in this task file

## Acceptance Criteria

- [ ] POST request with JSON body is forwarded successfully
- [ ] Content-Type header is preserved through the tunnel
- [ ] Request body arrives at local service intact
- [ ] Server logs show POST method correctly
- [ ] Client logs show successful forwarding
- [ ] Empty POST bodies are handled correctly
- [ ] Special characters and Unicode in JSON are preserved
- [ ] Larger JSON payloads work without corruption
- [ ] No base64 encoding/decoding errors in logs

## Reference

See CLAUDE.md sections:
- "Test Case 3: POST with Body" (lines 581-595)
- "Acceptance Criteria - Arbitrary Paths and Queries" (lines 905-908)
