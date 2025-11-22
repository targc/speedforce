use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io;

/// Represents an HTTP request being forwarded from server to client through the tunnel.
///
/// The server receives an HTTP request and converts it into this format for transmission
/// over the TCP tunnel connection. The body is base64-encoded to support binary data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TunnelRequest {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,

    /// Full path including query string (e.g., "/api/v1/webhook?x=1")
    pub path: String,

    /// Header name-value pairs
    pub headers: Vec<(String, String)>,

    /// Base64-encoded body bytes (supports binary data)
    pub body: String,
}

/// Represents an HTTP response being sent from client back to server through the tunnel.
///
/// The client receives a response from the local HTTP service and converts it into this
/// format for transmission back to the server. The body is base64-encoded to support binary data.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TunnelResponse {
    /// HTTP status code (200, 404, 500, etc.)
    pub status: u16,

    /// Header name-value pairs
    pub headers: Vec<(String, String)>,

    /// Base64-encoded body bytes (supports binary data)
    pub body: String,
}

/// Writes a length-prefixed frame to a writer.
///
/// Frame format: [4 bytes: u32 big-endian length][N bytes: payload]
///
/// # Arguments
/// * `writer` - The async writer to write to
/// * `payload` - The bytes to write
///
/// # Returns
/// * `Ok(())` on success
/// * `Err` if writing fails
pub async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    payload: &[u8]
) -> io::Result<()> {
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

/// Reads a length-prefixed frame from a reader.
///
/// Frame format: [4 bytes: u32 big-endian length][N bytes: payload]
///
/// # Arguments
/// * `reader` - The async reader to read from
///
/// # Returns
/// * `Ok(Vec<u8>)` containing the payload on success
/// * `Err` if reading fails or connection is closed
pub async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R
) -> io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload).await?;
    Ok(payload)
}

/// Encodes binary body bytes as base64 string.
///
/// # Arguments
/// * `body_bytes` - The raw bytes to encode
///
/// # Returns
/// * Base64-encoded string
pub fn encode_body(body_bytes: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(body_bytes)
}

/// Decodes base64 string to binary body bytes.
///
/// # Arguments
/// * `encoded` - The base64-encoded string
///
/// # Returns
/// * `Ok(Vec<u8>)` containing decoded bytes on success
/// * `Err` if decoding fails
pub fn decode_body(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.decode(encoded)
}
