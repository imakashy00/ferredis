use std::{ fmt, io };

use crate::protocol::Frame;

#[derive(Debug)]
pub enum ProtocolError {
    // Invalid frame byte !in [+,-,:,$,*]
    InvalidType(u8),
    // Invalid utf8
    InvalidUtf8,
    // length field is not num or -ve other than -1
    InvalidLength,
    // Length too large for configured limit
    LengthTooLarge,
    // Missing terminator CRLF
    MissingTerminator,
    // Invalid int
    InvalidInteger,
    // Not array of bulk
    ExpectedBulkArray,
    // client's read buffer grew past the per-client limit
    QueryTooLarge,
    // Read - write failure
    Io(io::Error),
}

impl ProtocolError {
    pub fn to_frame(&self) -> Frame {
        Frame::Error(format!("Err Protocal: {}", self))
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::InvalidType(b) =>
                // escape default help printing the char without getting converted like \n, \t
                write!(f, "invalid byte {}", (*b as char).escape_default()),
            ProtocolError::InvalidUtf8 => write!(f, "invalidutf8"),
            ProtocolError::InvalidLength => write!(f, "invalid length"),
            ProtocolError::LengthTooLarge => write!(f, "length exceeds the allowed maximum"),
            ProtocolError::InvalidInteger => write!(f, "invalid integer"),
            ProtocolError::ExpectedBulkArray => write!(f, "expected array of bulk strings"),
            ProtocolError::MissingTerminator => write!(f, "expected CRLF terminator"),
            ProtocolError::QueryTooLarge => write!(f, "query buffer limit exceeded"),
            ProtocolError::Io(e) => write!(f, "io error {}", e),
        }
    }
}

// The source() function is used to chain errors together,
// allowing me to discover the underlying root cause of an error.
impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProtocolError::Io(e) => Some(e),
            _ => None,
        }
    }
}

// Lets the connection loop use `?` on socket operations.

// The impl From<io::Error> for ProtocolError block implements Rust's standard From trait.
// It explicitly defines a recipe for how to automatically convert a low-level std::io::Error into my custom ProtocolError::Io variant.
impl From<io::Error> for ProtocolError {
    fn from(e: io::Error) -> Self {
        ProtocolError::Io(e)
    }
}
