//! Wire format of the Arrow Native endpoint.
//!
//! Every message is a frame: a big-endian `u32` payload length, then the
//! payload, whose first byte is the message type. Strings and byte blobs are a
//! `u32` length followed by the data; an optional string is a `u8` presence
//! flag followed by the string. Schema and batch payloads are self-contained
//! Arrow IPC streams.
//!
//! A session is: handshake, authentication, then any number of queries. Each
//! query is answered by a schema, zero or more batches and a completion, or by
//! an error at any point. An error ends the response but not the connection.

use crate::CubeError;
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Version 1: authentication sends a token, used as the SQL API password.
pub const PROTOCOL_VERSION_1: u32 = 1;
/// Version 2: authentication also sends an optional user name, so the
/// endpoint authenticates exactly like the Postgres endpoint does.
pub const PROTOCOL_VERSION_2: u32 = 2;

pub fn is_supported_version(version: u32) -> bool {
    matches!(version, PROTOCOL_VERSION_1 | PROTOCOL_VERSION_2)
}

/// Largest frame accepted before the client has authenticated. Handshake and
/// authentication messages carry a version, a token and a few names.
pub const MAX_UNAUTHENTICATED_FRAME_LEN: u32 = 64 * 1024;
/// Largest frame accepted from an authenticated client (the query text).
pub const MAX_AUTHENTICATED_FRAME_LEN: u32 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    HandshakeRequest = 0x01,
    HandshakeResponse = 0x02,
    AuthRequest = 0x03,
    AuthResponse = 0x04,
    QueryRequest = 0x10,
    QueryResponseSchema = 0x11,
    QueryResponseBatch = 0x12,
    QueryComplete = 0x13,
    Error = 0xFF,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Result<Self, CubeError> {
        match value {
            0x01 => Ok(MessageType::HandshakeRequest),
            0x02 => Ok(MessageType::HandshakeResponse),
            0x03 => Ok(MessageType::AuthRequest),
            0x04 => Ok(MessageType::AuthResponse),
            0x10 => Ok(MessageType::QueryRequest),
            0x11 => Ok(MessageType::QueryResponseSchema),
            0x12 => Ok(MessageType::QueryResponseBatch),
            0x13 => Ok(MessageType::QueryComplete),
            0xFF => Ok(MessageType::Error),
            _ => Err(CubeError::user(format!(
                "Unknown message type: 0x{:02x}",
                value
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    HandshakeRequest {
        version: u32,
    },
    HandshakeResponse {
        version: u32,
        server_version: String,
    },
    /// `user` is only on the wire in protocol version 2.
    AuthRequest {
        user: Option<String>,
        password: String,
        database: Option<String>,
    },
    AuthResponse {
        success: bool,
        session_id: String,
    },
    QueryRequest {
        sql: String,
    },
    QueryResponseSchema {
        arrow_ipc_schema: Vec<u8>,
    },
    QueryResponseBatch {
        arrow_ipc_batch: Vec<u8>,
    },
    QueryComplete {
        rows_affected: i64,
    },
    Error {
        code: String,
        message: String,
    },
}

impl Message {
    /// Encodes the message as a complete frame, length prefix included.
    pub fn encode(&self, version: u32) -> Vec<u8> {
        let mut buf = BytesMut::new();

        match self {
            Message::HandshakeRequest { version } => {
                buf.put_u8(MessageType::HandshakeRequest as u8);
                buf.put_u32(*version);
            }
            Message::HandshakeResponse {
                version,
                server_version,
            } => {
                buf.put_u8(MessageType::HandshakeResponse as u8);
                buf.put_u32(*version);
                put_string(&mut buf, server_version);
            }
            Message::AuthRequest {
                user,
                password,
                database,
            } => {
                buf.put_u8(MessageType::AuthRequest as u8);
                if version >= PROTOCOL_VERSION_2 {
                    put_optional_string(&mut buf, user.as_deref());
                }
                put_string(&mut buf, password);
                put_optional_string(&mut buf, database.as_deref());
            }
            Message::AuthResponse {
                success,
                session_id,
            } => {
                buf.put_u8(MessageType::AuthResponse as u8);
                buf.put_u8(u8::from(*success));
                put_string(&mut buf, session_id);
            }
            Message::QueryRequest { sql } => {
                buf.put_u8(MessageType::QueryRequest as u8);
                put_string(&mut buf, sql);
            }
            Message::QueryResponseSchema { arrow_ipc_schema } => {
                buf.put_u8(MessageType::QueryResponseSchema as u8);
                put_bytes(&mut buf, arrow_ipc_schema);
            }
            Message::QueryResponseBatch { arrow_ipc_batch } => {
                buf.put_u8(MessageType::QueryResponseBatch as u8);
                put_bytes(&mut buf, arrow_ipc_batch);
            }
            Message::QueryComplete { rows_affected } => {
                buf.put_u8(MessageType::QueryComplete as u8);
                buf.put_i64(*rows_affected);
            }
            Message::Error { code, message } => {
                buf.put_u8(MessageType::Error as u8);
                put_string(&mut buf, code);
                put_string(&mut buf, message);
            }
        }

        let mut frame = Vec::with_capacity(4 + buf.len());
        frame.put_u32(buf.len() as u32);
        frame.extend_from_slice(&buf);
        frame
    }

    /// Decodes a frame payload (without the length prefix). Truncated or
    /// trailing data is an error, never a panic: this runs on unauthenticated
    /// input.
    pub fn decode(payload: &[u8], version: u32) -> Result<Self, CubeError> {
        let mut buf = payload;
        let msg_type = MessageType::from_u8(get_u8(&mut buf)?)?;

        let message = match msg_type {
            MessageType::HandshakeRequest => Message::HandshakeRequest {
                version: get_u32(&mut buf)?,
            },
            MessageType::HandshakeResponse => Message::HandshakeResponse {
                version: get_u32(&mut buf)?,
                server_version: get_string(&mut buf)?,
            },
            MessageType::AuthRequest => {
                let user = if version >= PROTOCOL_VERSION_2 {
                    get_optional_string(&mut buf)?
                } else {
                    None
                };
                Message::AuthRequest {
                    user,
                    password: get_string(&mut buf)?,
                    database: get_optional_string(&mut buf)?,
                }
            }
            MessageType::AuthResponse => Message::AuthResponse {
                success: get_u8(&mut buf)? != 0,
                session_id: get_string(&mut buf)?,
            },
            MessageType::QueryRequest => Message::QueryRequest {
                sql: get_string(&mut buf)?,
            },
            MessageType::QueryResponseSchema => Message::QueryResponseSchema {
                arrow_ipc_schema: get_bytes(&mut buf)?,
            },
            MessageType::QueryResponseBatch => Message::QueryResponseBatch {
                arrow_ipc_batch: get_bytes(&mut buf)?,
            },
            MessageType::QueryComplete => Message::QueryComplete {
                rows_affected: get_i64(&mut buf)?,
            },
            MessageType::Error => Message::Error {
                code: get_string(&mut buf)?,
                message: get_string(&mut buf)?,
            },
        };

        if buf.has_remaining() {
            return Err(CubeError::user(format!(
                "Unexpected {} trailing bytes in {:?} message",
                buf.remaining(),
                msg_type
            )));
        }

        Ok(message)
    }
}

fn put_string(buf: &mut BytesMut, s: &str) {
    put_bytes(buf, s.as_bytes());
}

fn put_optional_string(buf: &mut BytesMut, s: Option<&str>) {
    match s {
        Some(s) => {
            buf.put_u8(1);
            put_string(buf, s);
        }
        None => buf.put_u8(0),
    }
}

fn put_bytes(buf: &mut BytesMut, bytes: &[u8]) {
    buf.put_u32(bytes.len() as u32);
    buf.put_slice(bytes);
}

fn ensure(buf: &[u8], len: usize) -> Result<(), CubeError> {
    if buf.remaining() < len {
        return Err(CubeError::user("Truncated message".to_string()));
    }
    Ok(())
}

fn get_u8(buf: &mut &[u8]) -> Result<u8, CubeError> {
    ensure(buf, 1)?;
    Ok(buf.get_u8())
}

fn get_u32(buf: &mut &[u8]) -> Result<u32, CubeError> {
    ensure(buf, 4)?;
    Ok(buf.get_u32())
}

fn get_i64(buf: &mut &[u8]) -> Result<i64, CubeError> {
    ensure(buf, 8)?;
    Ok(buf.get_i64())
}

fn get_bytes(buf: &mut &[u8]) -> Result<Vec<u8>, CubeError> {
    let len = get_u32(buf)? as usize;
    ensure(buf, len)?;
    let bytes = buf[..len].to_vec();
    buf.advance(len);
    Ok(bytes)
}

fn get_string(buf: &mut &[u8]) -> Result<String, CubeError> {
    String::from_utf8(get_bytes(buf)?)
        .map_err(|e| CubeError::user(format!("Invalid UTF-8 string: {}", e)))
}

fn get_optional_string(buf: &mut &[u8]) -> Result<Option<String>, CubeError> {
    match get_u8(buf)? {
        0 => Ok(None),
        _ => Ok(Some(get_string(buf)?)),
    }
}

/// Reads one frame and decodes it, refusing frames longer than `max_len`
/// before allocating for them. Returns `None` when the peer closed the
/// connection between frames.
pub async fn read_message<R: AsyncReadExt + Unpin>(
    reader: &mut R,
    version: u32,
    max_len: u32,
) -> Result<Option<Message>, CubeError> {
    let len = match reader.read_u32().await {
        Ok(len) => len,
        Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(err) => return Err(err.into()),
    };

    if len == 0 {
        return Err(CubeError::user("Invalid message length: 0".to_string()));
    }

    if len > max_len {
        return Err(CubeError::user(format!(
            "Message of {} bytes exceeds the limit of {} bytes",
            len, max_len
        )));
    }

    let mut payload = vec![0u8; len as usize];
    reader.read_exact(&mut payload).await?;

    Message::decode(&payload, version).map(Some)
}

pub async fn write_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    message: &Message,
    version: u32,
) -> Result<(), CubeError> {
    writer.write_all(&message.encode(version)).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    fn roundtrip(msg: Message, version: u32) -> Message {
        let frame = msg.encode(version);
        let len = u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize;
        assert_eq!(len, frame.len() - 4);
        Message::decode(&frame[4..], version).unwrap()
    }

    #[test]
    fn test_roundtrip_all_messages() {
        let messages = vec![
            Message::HandshakeRequest { version: 1 },
            Message::HandshakeResponse {
                version: 2,
                server_version: "1.0.0".to_string(),
            },
            Message::AuthResponse {
                success: true,
                session_id: "42".to_string(),
            },
            Message::QueryRequest {
                sql: "SELECT 1".to_string(),
            },
            Message::QueryResponseSchema {
                arrow_ipc_schema: vec![1, 2, 3],
            },
            Message::QueryResponseBatch {
                arrow_ipc_batch: vec![],
            },
            Message::QueryComplete { rows_affected: -1 },
            Message::Error {
                code: "QUERY_ERROR".to_string(),
                message: "no such cube".to_string(),
            },
        ];

        for msg in messages {
            assert_eq!(roundtrip(msg.clone(), PROTOCOL_VERSION_1), msg);
        }
    }

    #[test]
    fn test_byte_layout_of_query_request() {
        let frame = Message::QueryRequest {
            sql: "SELECT".to_string(),
        }
        .encode(PROTOCOL_VERSION_1);

        assert_eq!(frame, b"\0\0\0\x0b\x10\0\0\0\x06SELECT".to_vec());
    }

    #[test]
    fn test_auth_request_v1_has_no_user() {
        let msg = Message::AuthRequest {
            user: None,
            password: "token".to_string(),
            database: Some("db".to_string()),
        };
        assert_eq!(roundtrip(msg.clone(), PROTOCOL_VERSION_1), msg);

        // A v1 client's bytes: token, then the optional database
        let frame = msg.encode(PROTOCOL_VERSION_1);
        assert_eq!(&frame[4..10], b"\x03\0\0\0\x05t");
    }

    #[test]
    fn test_auth_request_v2_carries_user() {
        let msg = Message::AuthRequest {
            user: Some("cube".to_string()),
            password: "secret".to_string(),
            database: None,
        };
        assert_eq!(roundtrip(msg.clone(), PROTOCOL_VERSION_2), msg);
    }

    #[test]
    fn test_truncated_messages_are_errors() {
        let frame = Message::QueryRequest {
            sql: "SELECT 1".to_string(),
        }
        .encode(PROTOCOL_VERSION_1);
        let payload = &frame[4..];

        for len in 0..payload.len() {
            assert!(
                Message::decode(&payload[..len], PROTOCOL_VERSION_1).is_err(),
                "a {} byte prefix decoded",
                len
            );
        }

        // A string length pointing past the end of the payload
        assert!(Message::decode(b"\x10\xff\xff\xff\xffSELECT", PROTOCOL_VERSION_1).is_err());
    }

    #[test]
    fn test_trailing_bytes_are_an_error() {
        assert!(Message::decode(b"\x01\0\0\0\x01\0", PROTOCOL_VERSION_1).is_err());
    }

    #[test]
    fn test_unknown_message_type_is_an_error() {
        assert!(Message::decode(b"\x42", PROTOCOL_VERSION_1).is_err());
    }

    #[tokio::test]
    async fn test_read_message_refuses_oversized_frames_before_reading_them() {
        // Only the length prefix is present: the limit must be checked before
        // the payload is read (or allocated)
        let mut input: &[u8] = &(MAX_UNAUTHENTICATED_FRAME_LEN + 1).to_be_bytes();
        let err = read_message(
            &mut input,
            PROTOCOL_VERSION_1,
            MAX_UNAUTHENTICATED_FRAME_LEN,
        )
        .await
        .unwrap_err();
        assert!(err.message.contains("exceeds the limit"), "{}", err.message);
    }

    #[tokio::test]
    async fn test_read_write_message() {
        let msg = Message::QueryComplete { rows_affected: 3 };
        let mut buf = Vec::new();
        write_message(&mut buf, &msg, PROTOCOL_VERSION_1)
            .await
            .unwrap();

        let mut input: &[u8] = &buf;
        let read = read_message(&mut input, PROTOCOL_VERSION_1, MAX_AUTHENTICATED_FRAME_LEN)
            .await
            .unwrap();
        assert_eq!(read, Some(msg));

        // The input is exhausted at a frame boundary: a clean close
        let read = read_message(&mut input, PROTOCOL_VERSION_1, MAX_AUTHENTICATED_FRAME_LEN)
            .await
            .unwrap();
        assert_eq!(read, None);
    }

    #[tokio::test]
    async fn test_eof_inside_a_frame_is_an_error() {
        let mut input: &[u8] = b"\0\0\0\x05\x10";
        assert!(
            read_message(&mut input, PROTOCOL_VERSION_1, MAX_AUTHENTICATED_FRAME_LEN)
                .await
                .is_err()
        );
    }
}
