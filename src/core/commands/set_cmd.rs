use std::{
    cmp::{Ordering, min},
    collections::HashSet,
};

use bytes::Bytes;

use crate::{
    core::{
        commands::{fetch_entry_with_key, now_secs},
        keyspace::{DataStruct, Entry, Ferredis},
    },
    protocol::Frame,
};

pub fn handle_sadd(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db).map(|entry| matches!(entry.value, DataStruct::Set(_))) {
        Some(false) => {
            return Frame::Error("Wrongtype".into());
        }
        Some(true) => {}
        None => {
            db.store.insert(
                key.clone(),
                Entry {
                    value: DataStruct::Set(HashSet::new()),
                    expires_at: None,
                    last_access: now_secs(),
                    approx_size: size_of_val(key),
                },
            );
        }
    }
    let Some(entry) = fetch_entry_with_key(key, db) else {
        return Frame::Error("Something went wrong".into());
    };
    let DataStruct::Set(set) = &mut entry.value else {
        return Frame::Error("Wrongtype".into());
    };
    let mut added_elements = 0usize;
    let mut added_size = 0usize;
    for value in &args[2..] {
        if set.insert(value.clone()) {
            added_elements += 1;
            added_size += size_of_val(value);
        }
    }
    entry.last_access = now_secs();
    entry.approx_size += added_size;
    db.current_memory_usage += added_size;
    Frame::Integer(added_elements as i64)
}
pub fn handle_srem(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // more than one element can be remove ignore the non existing
    let key = &args[1];
    let values = &args[2..];
    match fetch_entry_with_key(key, db) {
        None => Frame::Integer(0),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Integer(0);
            };
            let mut removed = 0usize;
            let mut memory_freed = 0usize;
            for value in values {
                if set.remove(value) {
                    removed += 1;
                    memory_freed += size_of_val(value);
                }
            }
            entry.approx_size -= memory_freed;
            db.current_memory_usage -= memory_freed;
            return Frame::Integer(removed as i64);
        }
    }
}
pub fn handle_sismember(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    let value = &args[2];
    match fetch_entry_with_key(key, db) {
        None => Frame::Integer(0),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Integer(0);
            };
            Frame::Integer(set.contains(value) as i64)
        }
    }
}
pub fn handle_smembers(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // return all members
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        None => Frame::Array(None),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Array(None);
            };
            let mut members = Vec::new();
            for member in set.iter() {
                members.push(Frame::Bulk(Some(member.clone())));
            }
            Frame::Array(Some(members))
        }
    }
}
pub fn handle_scard(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let key = &args[1];
    match fetch_entry_with_key(key, db) {
        None => Frame::Integer(0),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Integer(0);
            };
            Frame::Integer(set.len() as i64)
        }
    }
}
pub fn handle_spop(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // pops minimum one else given number of elements form the set
    let key = &args[1];
    if !(2..=3).contains(&args.len()) {
        return Frame::Error("Err: Wrong number of arguments".into());
    }
    let mut count = match args.get(2) {
        Some(value) => match String::from_utf8_lossy(value).parse::<usize>() {
            Ok(val) => val,
            Err(_) => {
                return Frame::Error("Failed to parse".into());
            }
        },
        None => 1,
    };
    match fetch_entry_with_key(key, db) {
        None => Frame::Bulk(None),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Integer(0);
            };
            if count == 1 {
                if let Some(member) = set.iter().next() {
                    let val = member.clone();
                    if let Some(removed) = set.take(&val) {
                        return Frame::Bulk(Some(removed));
                    }
                }
            }
            let mut removed_ele = Vec::new();
            while count > 0 {
                match set.iter().next().cloned() {
                    Some(member) => {
                        if let Some(removed) = set.take(&member) {
                            removed_ele.push(Frame::Bulk(Some(removed)));
                        }
                    }
                    None => {
                        break;
                    }
                }
                count -= 1;
            }
            Frame::Array(Some(removed_ele))
        }
    }
}
pub fn handle_srandmember(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Returns random member if no count
    let key = &args[1];
    if !(2..=3).contains(&args.len()) {
        return Frame::Error("Err: Wrong number of arguments".into());
    }
    let mut count = match args.get(2) {
        Some(value) => match String::from_utf8_lossy(value).parse::<isize>() {
            Ok(val) => val,
            Err(_) => {
                return Frame::Error("Failed to parse".into());
            }
        },
        None => 1,
    };
    match fetch_entry_with_key(key, db) {
        None => Frame::Bulk(None),
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            let mut members = Vec::new();
            match count.cmp(&1) {
                Ordering::Equal | Ordering::Greater => {
                    // if count is +ve then min(cardinal, count) members
                    let mut rand_members = min(count, set.len() as isize);
                    while rand_members > 0 {
                        match set.iter().next() {
                            Some(member) => {
                                members.push(Frame::Bulk(Some(member.clone())));
                            }
                            None => {
                                break;
                            }
                        }
                        rand_members -= 1;
                    }
                    Frame::Array(Some(members))
                }
                // if count is -ve then atleast count members if s-card is less then repeat
                Ordering::Less => {
                    // cyclic iter
                    let mut cyclic_iter = set.iter().cycle();
                    while count < 0 {
                        match cyclic_iter.next() {
                            Some(member) => {
                                members.push(Frame::Bulk(Some(member.clone())));
                            }
                            None => {
                                break;
                            }
                        }
                        count += 1;
                    }
                    Frame::Array(Some(members))
                }
            }
        }
    }
}
pub fn handle_sunion(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // takes list of set keys and return union of them
    let keys = &args[1..];
    let mut union = HashSet::new();
    for key in keys {
        match fetch_entry_with_key(key, db) {
            // if Entry(key) is present
            Some(entry) => {
                let DataStruct::Set(set) = &mut entry.value else {
                    return Frame::Bulk(None);
                };
                for member in set.iter() {
                    union.insert(member.clone());
                }
            }
            // if Entry is absent
            None => {
                continue;
            }
        }
    }
    let frames = union
        .into_iter()
        .map(|frame| Frame::Bulk(Some(frame)))
        .collect::<Vec<Frame>>();
    Frame::Array(Some(frames))
}
pub fn handle_sinter(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // takes list of set keys and return intersection of them
    let keys = &args[1..];

    // get the first key entry
    let mut intersection: Vec<Bytes> = match fetch_entry_with_key(&keys[0], db) {
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            // clone and collect the members
            set.iter().cloned().collect()
        }
        None => {
            return Frame::Array(None);
        }
    };

    for key in &keys[1..] {
        if intersection.is_empty() {
            // first set is empty and set() & set(...)= set()
            break;
        }
        match fetch_entry_with_key(key, db) {
            // if Entry(key) is present
            Some(entry) => {
                let DataStruct::Set(set) = &mut entry.value else {
                    return Frame::Bulk(None);
                };
                // remove every element from the intersection that is not present in other sets
                intersection.retain(|member| set.contains(member));
            }
            // if Entry is absent
            None => {
                return Frame::Array(None);
            }
        }
    }
    let frames = intersection
        .into_iter()
        .map(|frame| Frame::Bulk(Some(frame)))
        .collect::<Vec<Frame>>();
    Frame::Array(Some(frames))
}
pub fn handle_sdiff(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Returns the members of the set resulting from the difference between the first set and all the successive sets.
    let keys = &args[1..];
    // get the first key entry
    let mut diff: Vec<Bytes> = match fetch_entry_with_key(&keys[0], db) {
        Some(entry) => {
            let DataStruct::Set(set) = &mut entry.value else {
                return Frame::Bulk(None);
            };
            // clone and collect the members
            set.iter().cloned().collect()
        }
        None => {
            return Frame::Array(None);
        }
    };
    for key in &keys[1..] {
        if diff.is_empty() {
            // first set is empty and set() & set(...)= set()
            break;
        }
        match fetch_entry_with_key(key, db) {
            // if Entry(key) is present
            Some(entry) => {
                let DataStruct::Set(set) = &mut entry.value else {
                    return Frame::Bulk(None);
                };
                // remove every element from the intersection that is not present in other sets
                diff.retain(|member| !set.contains(member));
            }
            // if Entry is absent
            None => {
                continue;
            }
        }
    }
    let frames = diff
        .into_iter()
        .map(|frame| Frame::Bulk(Some(frame)))
        .collect::<Vec<Frame>>();
    Frame::Array(Some(frames))
}
