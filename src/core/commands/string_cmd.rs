use bytes::Bytes;

use crate::{ core::{ keyspace::{ DataStruct, Ferredis }, types::fetch_with_key }, protocol::Frame };

pub fn handle_set(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_get(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];

    // fetch the value form the map
    match fetch_with_key(key, db) {
        Some(DataStruct::String(bytes)) => {
            return Frame::Bulk(Some(bytes.clone()));
        }
        Some(_) => {
            return Frame::Error("Wrong Operation".into());
        }
        None => {
            return Frame::Bulk(None);
        }
    }
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
