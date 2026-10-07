use std::fmt::{ Display, write };

use bytes::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub enum Frame {
    String(String),
    Integer(i64),
    Bulk(Option<Bytes>),
    Array(Option<Vec<Frame>>),
}

#[derive(Debug)]
pub struct Protocolerror(pub String);


