use bytes::Bytes;

use crate::{ core::keyspace::Ferredis, protocol::Frame };

pub fn handle_ping(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("PONG".into())
}
pub fn handle_echo(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    if args.len() == 2 {
        Frame::Bulk(Some(args[1].clone()))
    } else {
        Frame::Error("missing argument".into())
    }
}
pub fn handle_command(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("COMMAND".into())
}
pub fn handle_config(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("CONFIG".into())
}
pub fn handle_dbsize(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_flushall(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}

pub fn handle_save(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
pub fn handle_shutdown(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("DBSIZE".into())
}
