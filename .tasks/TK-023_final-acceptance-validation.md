# Task: Run Complete Acceptance Criteria Validation

**Status**: pending
**Dependencies**: TK-021, TK-022
**Estimated Effort**: small

## Objective

Systematically validate all acceptance criteria from CLAUDE.md to confirm the implementation is complete and correct.

## Context

This is the final validation task that goes through every acceptance criterion in the specification. We'll run through all test cases one more time to ensure nothing was broken during development and that the system meets all requirements before considering the project complete.

## Files to Modify/Create

No code changes - this is a comprehensive testing task

## Detailed Steps

1. Validate Basic Forwarding (AC #1):
   - [ ] Server running on 0.0.0.0:8080 (HTTP) and 0.0.0.0:7000 (TCP)
   - [ ] Client connected to server and forwarding to localhost:3000
   - [ ] HTTP request to server is forwarded to local service
   - [ ] Local service response is returned to original caller
   - [ ] Method, path, headers, body are preserved exactly

2. Validate Arbitrary Paths and Queries (AC #2):
   - [ ] Request to `/foo/bar?x=1&y=2` arrives at local service as `/foo/bar?x=1&y=2`
   - [ ] Special characters in query strings are preserved
   - [ ] POST requests with JSON bodies work correctly

3. Validate Client Down Scenario (AC #3):
   - [ ] When no client connected, server returns 503 Service Unavailable
   - [ ] Error message is clear and helpful
   - [ ] Server continues running and accepting HTTP requests

4. Validate Client Reconnect (AC #4):
   - [ ] Client can be stopped and restarted
   - [ ] Client reconnects automatically without manual intervention
   - [ ] After reconnection, requests flow normally
   - [ ] Server does NOT need to be restarted

5. Validate Two Clients / Last One Wins (AC #5):
   - [ ] When second client connects, first client is disconnected
   - [ ] Subsequent requests go only to the latest connected client
   - [ ] No split-brain behavior (requests never go to old client)
   - [ ] Server logs indicate client replacement

6. Validate Server Restart Recovery (AC #6):
   - [ ] Client survives server restarts
   - [ ] Client reconnects when server is back up
   - [ ] Exponential backoff prevents connection spam
   - [ ] Maximum backoff prevents excessive delays

7. Validate Binary Data Support (AC #7):
   - [ ] Binary request bodies (octet-stream) are transmitted correctly
   - [ ] Binary response bodies are transmitted correctly
   - [ ] No data corruption or truncation
   - [ ] Base64 encoding/decoding is transparent

8. Validate Error Handling (AC #8):
   - [ ] Local service down: Client returns 502 in tunnel response
   - [ ] Tunnel write failure: Server returns 502 to HTTP caller
   - [ ] Invalid protocol message: Server returns 502 to HTTP caller
   - [ ] All errors are logged with sufficient detail

9. Review logs for all test cases:
   - Verify structured logging is working
   - Verify info, debug, and error levels are appropriate
   - Verify no excessive or missing log output

10. Create final test report:
    - Document all passing criteria
    - Document any failures or issues
    - Note any deviations from spec

## Acceptance Criteria

- [ ] All 8 acceptance criteria sections from CLAUDE.md pass completely
- [ ] No regressions from previous tests
- [ ] All logging is appropriate and helpful
- [ ] No panics or crashes occur in any test case
- [ ] System behaves as specified under all conditions
- [ ] Test report is documented in this file

## Reference

See CLAUDE.md sections:
- "Acceptance Criteria" (lines 894-943)
- "Manual Testing Checklist" (lines 519-728)
