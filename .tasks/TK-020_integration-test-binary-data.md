# Task: Test Binary Data Transmission

**Status**: pending
**Dependencies**: TK-019
**Estimated Effort**: small

## Objective

Verify that binary data (non-text HTTP bodies) can be transmitted through the tunnel without corruption using base64 encoding.

## Context

Some webhooks or API requests include binary data (images, PDFs, compressed files, etc.). The tunnel must handle arbitrary binary data by base64-encoding it during transmission over the JSON protocol. This test ensures the encoding/decoding is correct and no data corruption occurs.

## Files to Modify/Create

No code changes - this is a manual testing task

## Detailed Steps

1. Create test binary file:
   - `dd if=/dev/urandom of=/tmp/test.bin bs=1024 count=10`
   - This creates a 10KB file with random binary data

2. Calculate checksum of original file:
   - `md5sum /tmp/test.bin` (Linux) or `md5 /tmp/test.bin` (macOS)
   - Record the hash for comparison

3. Set up a simple echo server that returns the POST body:
   - Create `/tmp/echo_server.py`:
     ```python
     from http.server import HTTPServer, BaseHTTPRequestHandler

     class EchoHandler(BaseHTTPRequestHandler):
         def do_POST(self):
             content_length = int(self.headers['Content-Length'])
             body = self.rfile.read(content_length)
             self.send_response(200)
             self.send_header('Content-Type', 'application/octet-stream')
             self.send_header('Content-Length', str(len(body)))
             self.end_headers()
             self.wfile.write(body)

     HTTPServer(('127.0.0.1', 3000), EchoHandler).serve_forever()
     ```
   - Terminal 1: `python3 /tmp/echo_server.py`

4. Start tunnel server and client:
   - Terminal 2: `cargo run --bin tunnel-server`
   - Terminal 3: `SERVER_ADDR=127.0.0.1:7000 LOCAL_PORT=3000 cargo run --bin tunnel-client`

5. Send binary data through tunnel:
   - Terminal 4: `curl -X POST -H "Content-Type: application/octet-stream" --data-binary @/tmp/test.bin http://localhost:8080/upload -o /tmp/test_response.bin`
   - This sends the binary file and saves the echoed response

6. Verify data integrity:
   - Calculate checksum of response: `md5sum /tmp/test_response.bin` or `md5 /tmp/test_response.bin`
   - Compare with original hash - should be IDENTICAL
   - If hashes match, binary data was transmitted without corruption

7. Test with different binary file types:
   - Test with a small PNG image (if available): `curl -X POST --data-binary @image.png http://localhost:8080/image -o /tmp/response.png`
   - Open response.png to verify it's not corrupted

8. Test with empty binary body:
   - `curl -X POST -H "Content-Type: application/octet-stream" --data-binary @/dev/null http://localhost:8080/empty`
   - Verify request completes without errors

9. Check logs for any base64 encoding/decoding errors

10. Document results and hash comparisons

## Acceptance Criteria

- [ ] Binary file can be sent through tunnel via POST
- [ ] Response body matches original file byte-for-byte (checksums match)
- [ ] No data corruption or truncation occurs
- [ ] Content-Type header is preserved
- [ ] Base64 encoding/decoding is transparent to the client
- [ ] Empty binary bodies are handled correctly
- [ ] No encoding/decoding errors appear in logs
- [ ] Different binary file types work correctly

## Reference

See CLAUDE.md sections:
- "Test Case 7: Binary Data Handling" (lines 685-705)
- "Acceptance Criteria - Binary Data Support" (lines 930-934)
- "Binary Data Handling" (lines 116-121)
- "Base64 Body Encoding" (lines 431-444)
