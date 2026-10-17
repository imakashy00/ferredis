use std::collections::VecDeque;

use bytes::Bytes;

use crate::{
    core::{
        commands::{ fetch_entry_with_key, now_secs },
        keyspace::{ DataStruct, Entry, Ferredis },
    },
    protocol::Frame,
};

pub fn handle_lpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let values = &args[2..];
    // if key does not exist create one empty list
    match fetch_entry_with_key(key, db).map(|entry| matches!(entry.value, DataStruct::List(_))) {
        Some(false) => {
            // if the key holds a value that is not a list return Error
            return Frame::Error("wrongtype".into());
        }
        Some(true) => {}
        // not found
        None => {
            db.store.insert(key.clone(), Entry {
                value: DataStruct::List(VecDeque::new()),
                expires_at: None,
                last_access: now_secs(),
                approx_size: 32 + key.len(),
            });
        }
    }

    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    let mut added_size = 0;
    // map over the values form left to right after key and push them to the start of the list
    for value in values {
        added_size += value.len(); // TODO: calculate len correctly
        list.push_front(value.clone());
    }
    let new_len = list.len();
    entry.approx_size += added_size;

    // update db size
    db.current_memory_usage += added_size;
    // Success: Integer reply: the length of the list after the push operation.
    Frame::Integer(new_len as i64)
}
pub fn handle_rpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let values = &args[2..];
    // if key does not exist create one empty list
    match fetch_entry_with_key(key, db).map(|entry| matches!(entry.value, DataStruct::List(_))) {
        Some(false) => {
            // if the key holds a value that is not a list return Error
            return Frame::Error("wrongtype".into());
        }
        Some(true) => {}
        // not found
        None => {
            db.store.insert(key.clone(), Entry {
                value: DataStruct::List(VecDeque::new()),
                expires_at: None,
                last_access: now_secs(),
                approx_size: 32 + key.len(),
            });
        }
    }

    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    let mut added_size = 0;
    // map over the values form left to right after key and push them to the start of the list
    for value in values {
        added_size += value.len(); // TODO: calculate len correctly
        list.push_back(value.clone());
    }
    let new_len = list.len();
    entry.approx_size += added_size;

    // update db size
    db.current_memory_usage += added_size;
    // Success: Integer reply: the length of the list after the push operation.
    Frame::Integer(new_len as i64)
}
pub fn handle_lpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let value = &args[2];
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    // remove from left
    list.pop_front();
    let new_len = list.len();
    entry.approx_size -= value.len();

    // update dbsize
    db.current_memory_usage -= value.len();

    Frame::Integer(new_len as i64)
}
pub fn handle_rpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let value = &args[2];
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    // remove from left
    list.pop_back();
    let new_len = list.len();
    entry.approx_size -= value.len();

    // update dbsize
    db.current_memory_usage -= value.len();

    Frame::Integer(new_len as i64)
}
pub fn handle_lrange(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Array reply: a list of elements in the specified range, or an empty array if the key doesn't exist.
    // starts with zero
    // offset can be -ve number -1 = last element
    // start and stop both are inclusive and -ve number means start or end counts from the tail's side
    todo!()
}
pub fn handle_lindex(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_lset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_lrem(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
pub fn handle_ltrim(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    todo!()
}
