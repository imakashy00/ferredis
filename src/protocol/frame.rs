use std::fmt;

use bytes::{ BufMut, Bytes, BytesMut };

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    Simple(String),
    Integer(i64),
    Bulk(Option<Bytes>),
    Array(Option<Vec<Frame>>),
    Error(String),
}

impl Frame {
    // No consumeption of self only read hence &
    pub fn encode(&self, buffer: &mut BytesMut) {
        match self {
            Frame::Simple(s) => {
                buffer.put_u8(b'+');
                buffer.put_slice(s.as_bytes());
                buffer.put_slice(b"\r\n");
            }
            Frame::Integer(s) => {
                buffer.put_u8(b':');
                buffer.put_slice(s.to_string().as_bytes());
                buffer.put_slice(b"\r\n");
            }
            Frame::Bulk(None) => {
                buffer.put_slice(b"$-1\r\n");
            }
            Frame::Bulk(Some(s)) => {
                buffer.put_u8(b'$');
                buffer.put_slice(s.len().to_string().as_bytes());
                buffer.put_slice(b"\r\n");
                buffer.put_slice(s);
                buffer.put_slice(b"\r\n");
            }
            Frame::Array(None) => {
                buffer.put_slice(b"*-1\r\n");
            }
            Frame::Array(Some(items)) => {
                buffer.put_u8(b'*');
                buffer.put_slice(items.len().to_string().as_bytes());
                buffer.put_slice(b"\r\n");
                // recursion
                for item in items {
                    item.encode(buffer);
                }
            }
            Frame::Error(s) => {
                buffer.put_u8(b'-');
                buffer.put_slice(s.to_string().as_bytes());
                buffer.put_slice(b"\r\n");
            }
        }
    }
}

#[derive(Debug)]
pub struct ProtocolError(pub String);

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ProtocolError {}

pub fn error<T>(msg: impl Into<String>) -> Result<T, ProtocolError> {
    Err(ProtocolError(msg.into()))
}
