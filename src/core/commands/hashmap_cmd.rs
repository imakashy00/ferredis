use std::collections::HashMap;

use bytes::Bytes;

use crate::{
    core::{
        commands::{fetch_entry_with_key, now_secs},
        keyspace::{DataStruct, Entry, Ferredis},
    },
    protocol::Frame,
};
#[derive(Debug, PartialEq)]
enum GetType {
    Get,
    Mget,
}
fn handle_get(key: &Bytes, db: &mut Ferredis, fields: &[Bytes], get_type: GetType) -> Frame {
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            let mut frames = Vec::new();
            for field in fields {
                match hash.get(field) {
                    Some(value) => {
                        frames.push(Frame::Bulk(Some(value.clone())));
                    }
                    None => {
                        frames.push(Frame::Bulk(None));
                    }
                }
            }
            match get_type {
                GetType::Get => frames[0].clone(),
                GetType::Mget => Frame::Array(Some(frames)),
            }
        }
        // No entry found
        None => {
            return Frame::Bulk(None);
        }
    }
}

pub fn handle_hset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // key field value [field, value ...]
    if args.len() % 2 != 0 {
        return Frame::Error("Missing value".into());
    }
    let key = &args[1];
    // find entry with the key
    match fetch_entry_with_key(key, db).map(|entry| matches!(entry.value, DataStruct::Hash(_))) {
        // entry is not Hash
        Some(false) => {
            return Frame::Error("Wrongtype".into());
        }
        // entry is hashmap
        Some(true) => {}
        // No entry found
        None => {
            db.store.insert(
                key.clone(),
                Entry {
                    value: DataStruct::Hash(HashMap::new()),
                    expires_at: None,
                    last_access: now_secs(),
                    approx_size: size_of_val(&key), // TODO: calculate correctly
                },
            );
        }
    }
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    let DataStruct::Hash(hash) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    // loop over and add them to the hashmap
    let pairs = &args[2..];
    let mut added_size: usize = 0;
    let mut feilds_added: usize = 0;
    for kv in pairs.chunks_exact(2) {
        hash.insert(kv[0].clone(), kv[1].clone());
        added_size += size_of_val(&kv[0]) + size_of_val(&kv[1]);
        feilds_added += 1;
    }
    entry.approx_size += added_size;
    // update db size
    db.current_memory_usage += added_size;
    Frame::Integer(feilds_added as i64)
}
pub fn handle_hget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_get(&args[1], db, &args[2..=2], GetType::Get)
}
pub fn handle_hmget(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_get(&args[1], db, &args[2..], GetType::Mget)
}
pub fn handle_hexists(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let field = &args[2];
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            match hash.get(field) {
                Some(_) => {
                    return Frame::Integer(1);
                }
                None => {
                    return Frame::Integer(0);
                }
            }
        }
        // No entry found
        None => {
            return Frame::Bulk(None);
        }
    }
}
pub fn handle_hlen(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            return Frame::Integer(hash.keys().len() as i64);
        }
        // No entry found
        None => {
            return Frame::Integer(0);
        }
    }
}
pub fn handle_hgetall(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            let mut frames = Vec::new();
            for (field, value) in hash {
                frames.push(Frame::Bulk(Some(field.clone())));
                frames.push(Frame::Bulk(Some(value.clone())));
            }
            return Frame::Array(Some(frames));
        }
        // No entry found
        None => {
            return Frame::Array(None);
        }
    }
}
pub fn handle_hkeys(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            let mut frames = Vec::new();
            for field in hash.keys() {
                frames.push(Frame::Bulk(Some(field.clone())));
            }
            return Frame::Array(Some(frames));
        }
        // No entry found
        None => {
            return Frame::Array(None);
        }
    }
}
pub fn handle_hvals(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        // entry found
        Some(entry) => {
            // check for entry is Hash
            let DataStruct::Hash(hash) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            entry.last_access = now_secs();
            let mut frames = Vec::new();
            for value in hash.values() {
                frames.push(Frame::Bulk(Some(value.clone())));
            }
            return Frame::Array(Some(frames));
        }
        // No entry found
        None => {
            return Frame::Array(None);
        }
    }
}
