use bytes::Bytes;

use crate::protocol::Frame;

pub fn handle_ping(args: &[Bytes]) -> Frame {
    Frame::Simple("PONG".into())
}
pub fn handle_echo(args: &[Bytes]) -> Frame {
    if args.len() == 2 {
        Frame::Bulk(Some(args[1].clone()))
    } else {
        Frame::Error("missing argument".into())
    }
}
pub fn handle_command(args: &[Bytes]) -> Frame {
    Frame::Simple("COMMAND".into())
}
pub fn handle_config(args: &[Bytes]) -> Frame {
    Frame::Simple("CONFIG".into())
}
