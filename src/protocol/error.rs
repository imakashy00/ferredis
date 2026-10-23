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
    // Missing terminator CRLF
    MissingTerminator,
    // Invalid int
    InvalidInteger,
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
            ProtocolError::InvalidInteger => write!(f, "invalid integer"),
            ProtocolError::MissingTerminator => write!(f, "expected CRLF terminator"),
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
