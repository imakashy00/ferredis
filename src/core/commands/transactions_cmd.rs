use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_multi(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_exec(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
