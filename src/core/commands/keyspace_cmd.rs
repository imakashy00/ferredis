use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_exists(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_del(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_keys(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_ttl(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_expire(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_persist(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_rename(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
