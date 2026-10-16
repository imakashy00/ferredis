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
    let expire_at = match args.len() {
        3 => None,
        5 =>
            match String::from_utf8_lossy(&args[4]).as_ref().parse::<u64>() {
                Ok(ttl) => { Some(ttl) }
                Err(_) => {
                    return Frame::Error("Failed to parse ttl".into());
                }
            }
        _ => {
            return Frame::Error("Wrong number of arguments for set".into());
        }
    };
    if let Some(entry) = fetch_entry_with_key(key, db) {
        db.current_memory_usage -= entry.approx_size;
    }
    let approx_size = 32 + value.len();
    // store in db
    db.store.insert(key.clone(), Entry {
        value: DataStruct::String(value.clone()),
        expires_at: expire_at,
        last_access: now_secs(),
        approx_size: 32 + value.len(),
    });
    db.current_memory_usage += approx_size;
    return Frame::Simple("Ok".into());
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
    // bulk string reply if value is present
    // None if key does not exists or key value is not a String
    let key = &args[1];
    if let Some(value) = fetch_value_with_key(key, db) {
        match value {
            DataStruct::String(string_value) => {
                return Frame::Bulk(Some(string_value));
            }
            _ => {
                return Frame::Bulk(None);
            }
        }
    }
    Frame::Bulk(None)
}
pub fn handle_mset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Simple String reply :: bcz mset can't fail to set correct data
    // else Error for wrong key value pair

    let pairs = &args[1..];
    if pairs.len() % 2 != 0 {
        return Frame::Error("ERR wrong number of arguments for 'mset' command".into());
    }
    // iterate over all the key-value and insert them
    for kv in pairs.chunks_exact(2) {
        db.store.insert(kv[0].clone(), Entry {
            value: DataStruct::String(kv[1].clone()),
            expires_at: None,
            last_access: 0,
            approx_size: 32 + kv[1].len(),
        });
    }

    Frame::Simple("Ok".into())
}
pub fn handle_mget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Array reply : list of values at specified keys
    Frame::Array(
        Some(
            args
                .iter()
                .skip(1)
                .map(|key| {
                    match fetch_value_with_key(key, db) {
                        Some(value) => {
                            match value {
                                DataStruct::String(string_value) => {
                                    return Frame::Bulk(Some(string_value));
                                }
                                _ => {
                                    return Frame::Bulk(None);
                                }
                            }
                        }
                        None => {
                            return Frame::Bulk(None);
                        }
                    }
                })
                .collect()
        )
    )
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
