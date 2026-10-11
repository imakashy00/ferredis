use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };
// use std::io::Error;

// fn parse_lpush(args: Vec<Frame>) -> Result<(String, Vec<Vec<u8>>), Error> {
//     Ok(("sdskfls".into(), vec![b"32".to_vec()]))
// }
// fn execute_lpush(db: &mut Ferredis, key: String, values: Vec<Vec<u8>>) -> Frame {
//     Frame::Integer(5)
// }

pub fn handle_lpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_rpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_lpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_rpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_lrange(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_lindex(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_lset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_lrem(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_ltrim(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
