# Task: Define Protocol Data Structures

**Status**: pending
**Dependencies**: TK-001
**Estimated Effort**: small

## Objective

Implement TunnelRequest and TunnelResponse structs with Serde serialization/deserialization in the tunnel-protocol crate.

## Context

The tunnel protocol uses JSON-serialized messages to communicate between server and client. TunnelRequest represents HTTP requests being forwarded from server to client, while TunnelResponse represents HTTP responses being sent back from client to server. Both use base64-encoded strings for binary body data. These are the core data structures for the entire tunnel communication protocol.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-protocol/src/lib.rs` - Create protocol structs

## Detailed Steps

1. Create `tunnel-protocol/src/lib.rs` with module-level imports:
   - `use serde::{Deserialize, Serialize};`

2. Define `TunnelRequest` struct with derive macros `Serialize, Deserialize, Debug, Clone`:
   - `method: String` - HTTP method (GET, POST, etc.)
   - `path: String` - Full path with query string
   - `headers: Vec<(String, String)>` - Header name-value pairs
   - `body: String` - Base64-encoded body bytes

3. Define `TunnelResponse` struct with derive macros `Serialize, Deserialize, Debug, Clone`:
   - `status: u16` - HTTP status code
   - `headers: Vec<(String, String)>` - Header name-value pairs
   - `body: String` - Base64-encoded body bytes

4. Add public visibility to both structs and all fields

5. Add documentation comments to each struct and field explaining their purpose

6. Test compilation with `cargo check -p tunnel-protocol`

## Acceptance Criteria

- [ ] TunnelRequest struct is defined with all required fields
- [ ] TunnelResponse struct is defined with all required fields
- [ ] Both structs derive Serialize, Deserialize, Debug, Clone
- [ ] All fields are public and properly typed
- [ ] Headers use Vec<(String, String)> representation
- [ ] Body fields are String type (for base64 encoding)
- [ ] `cargo check -p tunnel-protocol` passes without errors
- [ ] Documentation comments are present and clear

## Reference

See CLAUDE.md sections:
- "Message Types" (lines 63-114)
- "Protocol Specification" (lines 49-128)
