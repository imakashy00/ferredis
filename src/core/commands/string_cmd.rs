use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_set(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("SET".into())
}
pub fn handle_get(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("SET".into())
}
