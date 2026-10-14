use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_set(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_get(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_get_del(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_mset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_mget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_incr(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_decr(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_incrby(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_decrby(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_strlen(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
