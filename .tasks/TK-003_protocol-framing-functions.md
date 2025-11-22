# Task: Implement Length-Prefixed Framing Functions

**Status**: pending
**Dependencies**: TK-002
**Estimated Effort**: small

## Objective

Implement write_frame and read_frame functions in tunnel-protocol for length-prefixed message framing over TCP connections.

## Context

All messages over the TCP tunnel use a simple framing format: [4 bytes: u32 big-endian length][N bytes: JSON payload]. This ensures message boundaries are preserved over the stream-oriented TCP connection. The framing functions are shared by both server and client for reading and writing protocol messages.

## Files to Modify/Create

- `/Users/tar/Documents/alpha/speedforce/tunnel-protocol/src/lib.rs` - Add framing functions

## Detailed Steps

1. Add imports to `lib.rs`:
   - `use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};`
   - `use std::io;`

2. Implement `pub async fn write_frame<W: AsyncWrite + Unpin>(writer: &mut W, payload: &[u8]) -> io::Result<()>`:
   - Calculate length as `payload.len() as u32`
   - Write 4-byte big-endian length prefix: `writer.write_all(&len.to_be_bytes()).await?`
   - Write payload bytes: `writer.write_all(payload).await?`
   - Flush writer: `writer.flush().await?`
   - Return Ok(())

3. Implement `pub async fn read_frame<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>>`:
   - Create 4-byte buffer: `let mut len_bytes = [0u8; 4];`
   - Read exactly 4 bytes: `reader.read_exact(&mut len_bytes).await?`
   - Parse length: `let len = u32::from_be_bytes(len_bytes) as usize;`
   - Create payload buffer: `let mut payload = vec![0u8; len];`
   - Read exact payload: `reader.read_exact(&mut payload).await?`
   - Return Ok(payload)

4. Add documentation comments explaining the framing format and usage

5. Test compilation with `cargo check -p tunnel-protocol`

## Acceptance Criteria

- [ ] write_frame function correctly writes length prefix in big-endian format
- [ ] write_frame flushes the writer after writing
- [ ] read_frame function correctly reads 4-byte length prefix
- [ ] read_frame parses length as big-endian u32
- [ ] read_frame allocates correct buffer size and reads exact payload
- [ ] Both functions use generic AsyncRead/AsyncWrite traits with Unpin bound
- [ ] Both functions return io::Result types
- [ ] Functions are public and well-documented
- [ ] `cargo check -p tunnel-protocol` passes without errors

## Reference

See CLAUDE.md sections:
- "Framing Format" (lines 51-61)
- "Length-Prefixed Framing" (lines 398-429)
