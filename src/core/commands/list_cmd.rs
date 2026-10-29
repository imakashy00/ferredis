use std::collections::VecDeque;

use bytes::Bytes;

use crate::{
    core::{
        commands::{fetch_entry_with_key, now_secs},
        keyspace::{DataStruct, Entry, Ferredis},
    },
    protocol::Frame,
};
#[derive(Debug, PartialEq)]
enum Direction {
    Left,
    Right,
}

fn handle_push(key: &Bytes, values: &[Bytes], push: Direction, db: &mut Ferredis) -> Frame {
    match fetch_entry_with_key(key, db).map(|entry| matches!(entry.value, DataStruct::List(_))) {
        Some(false) => {
            // if the key holds a value that is not a list return Error
            return Frame::Error("wrongtype".into());
        }
        Some(true) => {}
        // not found
        None => {
            db.store.insert(
                key.clone(),
                Entry {
                    value: DataStruct::List(VecDeque::new()),
                    expires_at: None,
                    last_access: now_secs(),
                    approx_size: 32 + key.len(),
                },
            );
        }
    }
    // if key does not exist create one empty list
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
        match push {
            Direction::Left => {
                list.push_front(value.clone());
            }
            Direction::Right => {
                list.push_back(value.clone());
            }
        }
    }
    let new_len = list.len();
    entry.approx_size += added_size;

    // update db size
    db.current_memory_usage += added_size;
    // Success: Integer reply: the length of the list after the push operation.
    Frame::Integer(new_len as i64)
}

fn handle_pop(args: &[Bytes], pop: Direction, db: &mut Ferredis) -> Frame {
    let key = &args[1];
    if !(2..=3).contains(&args.len()) {
        return Frame::Error("Err: Wrong number of arguments".into());
    }
    let mut count = match args.get(2) {
        Some(value) => match String::from_utf8_lossy(value).parse::<u32>() {
            Ok(val) => val,
            Err(_) => {
                return Frame::Error("Failed to parse".into());
            }
        },
        None => 1,
    };
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    // if count is grater then list then clear the list
    let mut reply_vec = Vec::new();
    let mut free_memory: usize = 0;
    // TODO: if count is one return Bulk not array!!!
    while count > 0 && !list.is_empty() {
        let popped = match pop {
            Direction::Left => list.pop_front(),
            Direction::Right => list.pop_back(),
        };
        match popped {
            Some(value) => {
                entry.approx_size -= value.len();
                free_memory += value.len();
                reply_vec.push(Frame::Bulk(Some(value))); // Removed .clone() if ownership allows
                count -= 1;
            }
            None => {
                break;
            }
        }
    }
    // update dbsize
    db.current_memory_usage -= free_memory;
    Frame::Array(Some(reply_vec))
}

pub fn handle_lpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_push(&args[1], &args[2..], Direction::Left, db)
}
pub fn handle_rpush(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_push(&args[1], &args[2..], Direction::Right, db)
}
pub fn handle_lpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_pop(args, Direction::Left, db)
}
pub fn handle_rpop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    handle_pop(args, Direction::Right, db)
}
pub fn handle_lrange(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Array reply: a list of elements in the specified range, or an empty array if the key doesn't exist.
    let key = &args[1];
    let mut start: i64 = match String::from_utf8_lossy(&args[2]).parse::<i64>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("WongIndex".into());
        }
    };
    let mut stop: i64 = match String::from_utf8_lossy(&args[3]).parse::<i64>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("WongIndex".into());
        }
    };

    // find if entry with this key exists
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    let len = list.len() as i64;

    // start and stop can +ve/-ve convert the ind to proper ind of the list
    start = if start < 0 { len + start } else { start };
    stop = if stop < 0 { len + stop } else { stop };

    // clamp
    if start < 0 {
        start = 0;
    }
    if stop >= len {
        stop = len - 1;
    }

    if start > stop || start < 0 || stop >= len {
        return Frame::Array(None);
    }
    let mut reply_vec = Vec::new();
    for ind in start..=stop {
        reply_vec.push(Frame::Bulk(Some(list[ind as usize].clone())));
    }
    Frame::Array(Some(reply_vec))
}
pub fn handle_lindex(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Nil reply: when index is out of range.
    // Bulk string reply: the requested element.
    let key = &args[1];
    let mut ind: i32 = match String::from_utf8_lossy(&args[2]).parse::<i32>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("WongIndex".into());
        }
    };
    // find if entry with this key exists
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };

    ind = if ind < 0 {
        (list.len() as i32) + ind
    } else {
        ind
    };

    if ind < 0 || ind > (list.len() as i32) {
        return Frame::Bulk(None);
    }
    Frame::Bulk(Some(list[ind as usize].clone()))
}
pub fn handle_lset(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Simple string reply: OK.
    let key = &args[1];
    let mut ind: i32 = match String::from_utf8_lossy(&args[2]).parse::<i32>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("WongIndex".into());
        }
    };
    let value = &args[3];
    // find if entry with this key exists
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    ind = if ind < 0 {
        (list.len() as i32) + ind
    } else {
        ind
    };
    if ind < 0 || ind > (list.len() as i32) {
        return Frame::Error("Index out of bound".into());
    }
    list[ind as usize] = value.clone();
    Frame::Simple("OK".into())
}
pub fn handle_lrem(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let count: i64 = match std::str::from_utf8(&args[2])
        .ok()
        .and_then(|s| s.parse().ok())
    {
        Some(n) => n,
        None => {
            return Frame::Error("ERR value is not an integer or out of range".into());
        }
    };
    let element = &args[3];
    // find if entry with this key exists
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Integer(0);
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };
    let before = list.len();
    let limit = count.unsigned_abs() as usize;

    if count == 0 {
        list.retain(|x| x != element);
    } else if count > 0 {
        let mut left = limit;
        list.retain(|x| {
            if left > 0 && x == element {
                left -= 1;
                false
            } else {
                true
            }
        });
    } else {
        let total = list.iter().filter(|x| *x == element).count();
        let mut to_keep = total.saturating_sub(limit); // earliest matches survive
        list.retain(|x| {
            if x != element {
                true
            } else if to_keep > 0 {
                to_keep -= 1;
                true
            } else {
                false
            }
        });
    }

    let removed = before - list.len();
    let now_empty = list.is_empty();

    let freed = removed * element.len();
    entry.approx_size = entry.approx_size.saturating_sub(freed);

    db.current_memory_usage = db.current_memory_usage.saturating_sub(freed);

    if now_empty {
        db.store.remove(key);
    }

    Frame::Integer(removed as i64)
}
pub fn handle_ltrim(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let start: i32 = match String::from_utf8_lossy(&args[2]).parse::<i32>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("ERR value is not an integer or out of range".into());
        }
    };
    let stop: i32 = match String::from_utf8_lossy(&args[3]).parse::<i32>() {
        Ok(num) => num,
        Err(_) => {
            return Frame::Error("ERR value is not an integer or out of range".into());
        }
    };

    // find if entry with this key exists
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Err: internal error".into());
    };
    // stored is List
    let DataStruct::List(list) = &mut entry.value else {
        return Frame::Error("WRONGTYPE".into());
    };

    let len = list.len() as i32;

    // start and stop can +ve/-ve convert the ind to proper ind of the list
    let mut start_idx = if start < 0 { len + start } else { start };
    let mut stop_idx = if stop < 0 { len + stop } else { stop };

    if start_idx >= len || start_idx > stop_idx {
        list.clear();
        return Frame::Simple("OK".into());
    }

    // clamp bound
    if start_idx < 0 {
        start_idx = 0;
    }
    if stop_idx >= len {
        stop_idx = len - 1;
    }

    let u_start = start_idx as usize;
    let u_stop = stop_idx as usize;

    // trim the list in-place using drain
    // remove trailing elements first so we don't shift the start index!
    if u_stop + 1 < list.len() {
        list.drain(u_stop + 1..);
    }
    // Remove leading elements
    list.drain(..u_start);

    // TODO! : Calculate the memory freed

    Frame::Simple("OK".into())
}
