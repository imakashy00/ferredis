use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_hset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hmget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hexists(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hlen(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hgetall(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hkeys(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_hvals(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
