use bytes::Bytes;

use crate::{core::keyspace::Ferredis, protocol::Frame};

pub fn handle_publish(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_subscribe(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
