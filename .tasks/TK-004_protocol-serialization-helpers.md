# Task: Add Protocol Serialization Helper Functions

**Status**: pending
**Dependencies**: TK-003
**Estimated Effort**: small

## Objective

Implement helper functions for serializing/deserializing protocol messages and encoding/decoding base64 body data in tunnel-protocol.

## Context

While TunnelRequest and TunnelResponse have Serde derive macros, we need convenient helper functions for the complete send/receive flow: serializing to JSON, writing framed messages, reading framed messages, and deserializing from JSON. We also need helpers for base64 encoding/decoding of HTTP body data to handle binary payloads safely.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-protocol/src/lib.rs` - Add helper functions

## Detailed Steps

1. Add imports:
   - `use base64::{Engine as _, engine::general_purpose::STANDARD};`

2. Implement base64 encoding helper:
   - `pub fn encode_body(body_bytes: &[u8]) -> String`
   - Return `STANDARD.encode(body_bytes)`
   - Handle empty body case (return empty string)

3. Implement base64 decoding helper:
   - `pub fn decode_body(encoded: &str) -> Result<Vec<u8>, base64::DecodeError>`
   - Handle empty string case (return empty vec)
   - Return `STANDARD.decode(encoded)`

4. Implement `pub async fn send_request<W: AsyncWrite + Unpin>(writer: &mut W, request: &TunnelRequest) -> io::Result<()>`:
   - Serialize request to JSON: `let json = serde_json::to_vec(request)?`
   - Convert serde_json::Error to io::Error using `map_err`
   - Call `write_frame(writer, &json).await`

5. Implement `pub async fn recv_request<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<TunnelRequest>`:
   - Call `read_frame(reader).await?` to get payload
   - Deserialize: `serde_json::from_slice(&payload)?`
   - Convert serde_json::Error to io::Error using `map_err`
   - Return deserialized TunnelRequest

6. Implement `pub async fn send_response<W: AsyncWrite + Unpin>(writer: &mut W, response: &TunnelResponse) -> io::Result<()>`:
   - Same pattern as send_request but for TunnelResponse

7. Implement `pub async fn recv_response<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<TunnelResponse>`:
   - Same pattern as recv_request but for TunnelResponse

8. Add comprehensive documentation for each helper function

9. Test compilation with `cargo check -p tunnel-protocol`

## Acceptance Criteria

- [ ] encode_body uses base64 STANDARD encoding
- [ ] decode_body returns Result with proper error type
- [ ] Empty body is handled correctly (empty string <-> empty vec)
- [ ] send_request serializes to JSON and writes framed message
- [ ] recv_request reads framed message and deserializes from JSON
- [ ] send_response and recv_response follow same pattern
- [ ] All serde_json::Error are properly converted to io::Error
- [ ] All functions are public and well-documented
- [ ] `cargo check -p tunnel-protocol` passes without errors

## Reference

See CLAUDE.md sections:
- "Binary Data Handling" (lines 116-121)
- "Base64 Body Encoding" (lines 431-444)
