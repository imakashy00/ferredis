use bytes::Bytes;

use crate::{
    core::{
        commands::keyspace_cmd::now_secs,
        keyspace::{ DataStruct, Entry, Ferredis },
        types::{ fetch_entry_with_key, fetch_value_with_key },
    },
    protocol::Frame,
};

pub fn handle_set(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let value = &args[2];
    // args must of length 3 or 5
    match args.len() {
        3 => {
            // find if key already in db repalce it and its expiration
            match fetch_entry_with_key(key, db) {
                Some(entry) => {
                    // reduce db size
                    db.current_memory_usage -= entry.approx_size;
                    // delete the available
                    db.store.remove(key);
                    // add the updated one
                    db.store.insert(key.clone(), Entry {
                        value: DataStruct::String(value.clone()),
                        expires_at: None,
                        last_access: now_secs(),
                        approx_size: 32 + value.len(),
                    });
                }
                None => {
                    // Add in db
                    db.store.insert(key.clone(), Entry {
                        value: DataStruct::String(value.clone()),
                        expires_at: None,
                        last_access: now_secs(),
                        approx_size: 32 + value.len(),
                    });
                }
            }
        }
        5 => {
            // Everything in 3 and expiry
            // let expire
        }
        _ => {
            return Frame::Error("Wrong number of arguments for set".into());
        }
    }
    todo!()
}
pub fn handle_get(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];

    // fetch the value form the map
    match fetch_value_with_key(key, db) {
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
