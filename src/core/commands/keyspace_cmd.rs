use std::{ time::{ SystemTime, UNIX_EPOCH } };

use bytes::Bytes;
use globset::GlobMatcher;

use crate::{ core::{ commands::now_secs, keyspace::Ferredis }, protocol::Frame };

pub fn handle_exists(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // count the number of key exists if found then return 1 else 0
    Frame::Integer(
        args
            .iter()
            .skip(1)
            .map(|key| db.store.contains_key(key))
            .count() as i64
    )
}
pub fn handle_del(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // deletes the existing keys in db else ignores and sends Frame::Integer for deleted keys
    Frame::Integer(
        args
            .iter()
            .skip(1)
            .map(|key| db.store.remove(key))
            .count() as i64
    )
}
pub fn handle_keys(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // convert the key pattern in to glob str
    let glob_str = String::from_utf8_lossy(&args[1]);

    // get a matcher
    let matcher: GlobMatcher = match globset::Glob::new(&glob_str) {
        Ok(g) => g.compile_matcher(),
        Err(_) => {
            return Frame::Error("ERR invalid pattern syntax".to_string());
        }
    };
    Frame::Array(
        Some(
            db.store
                .keys()
                // The '&*' derefs the Cow<str> into a &str, which implements AsRef<Path>.
                // is_match method take path that implments As Ref
                // '&*' takes a smart pointer wrapper, drills down to its inner value, and gives us a clean, standard borrow
                .filter(|key| matcher.is_match(&*String::from_utf8_lossy(key.as_ref().as_ref())))
                .map(|key| Frame::Bulk(Some(key.clone())))
                .collect()
        )
    )
}
pub fn handle_ttl(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // Returns the remaining time to live of a key that has a timeout.
    let key = &args[1];

    match db.store.get(key) {
        Some(entry) => {
            match entry.expires_at {
                Some(expire_at) => {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    if expire_at > now {
                        Frame::Integer((expire_at - now) as i64) // else (integer) seconds
                    } else {
                        Frame::Integer(-2)
                    }
                }
                None => Frame::Integer(-1), // -1 for no ttl set
            }
        }
        None => Frame::Integer(-2), // -2 for not exists
    }
}
pub fn handle_expire(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // A key with an associated timeout is often said to be 'volatile'
    // Set a timeout on key.
    // NX (Set the expiry only when the key has no expiry.)
    // XX (Set the expiry only when the key already has an expiry.)
    // GT (Set the expiry only when the new expiry is greater than the current one.)
    // LT (Set the expiry only when the new expiry is less than the current one.)
    // A non-volatile key is treated as an infinite TTL for the purpose of GT and LT.

    if !(3..=4).contains(&args.len()) {
        return Frame::Error("ERR wrong number of arguments for 'expire' command".into());
    }
    // check time
    let Some(secs) = std::str
        ::from_utf8(&args[2])
        .ok()
        .and_then(|s| s.parse::<i64>().ok()) else {
        return Frame::Error("ERR value is not an integer or out of range".into());
    };
    // no entry
    let Some(entry) = db.store.get_mut(&args[1]) else {
        return Frame::Integer(0);
    };

    let new_expire = now_secs().saturating_add_signed(secs);
    let cur_expire = entry.expires_at;
    let option = &args.get(3).map(|o| o.to_ascii_uppercase());
    let ok = match option.as_deref() {
        None => true, //change no condition
        Some(b"NX") => cur_expire.is_none(),
        Some(b"XX") => cur_expire.is_some(),
        Some(b"GT") => cur_expire.is_some_and(|c| new_expire > c),
        Some(b"LT") => cur_expire.is_none_or(|c| new_expire < c), // no exp so new is always less
        _ => {
            return Frame::Error("ERR Unsupported option".into());
        }
    };
    if ok {
        entry.expires_at = Some(new_expire);
    }
    Frame::Integer(ok as i64)
}

pub fn handle_persist(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let had_ttl = db.store.get_mut(&args[1]).is_some_and(|entry| entry.expires_at.take().is_some());
    Frame::Integer(had_ttl as i64)
}
pub fn handle_rename(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // HashMap hands out keys as &K because changing one would break its hash.
    // A rename has to be remove then insert.
    match db.store.remove(&args[1]) {
        Some(entry) => {
            db.store.insert(args[2].clone(), entry);
            Frame::Simple("OK".into())
        }
        None => {
            return Frame::Error("No such key found".into());
        }
    }
}
