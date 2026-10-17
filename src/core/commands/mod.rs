use std::time::{ SystemTime, UNIX_EPOCH };

use bytes::Bytes;

use crate::core::keyspace::{ DataStruct, Entry, Ferredis };

pub mod list_cmd;
pub mod hashmap_cmd;
pub mod set_cmd;
pub mod string_cmd;
pub mod keyspace_cmd;
pub mod server_cmd;
pub mod registry;
pub mod transactions_cmd;
pub mod pub_sub_cmd;

fn is_expired(entry: &Entry) -> bool {
    match entry.expires_at {
        Some(deadline) => deadline <= now_secs(),
        None => false,
    }
}
fn expire_if_needed(db: &mut Ferredis, key: &Bytes) {
    let expired = db.store.get(key).map_or(false, is_expired);
    if expired {
        if let Some(old) = db.store.remove(key) {
            db.current_memory_usage -= old.approx_size;
        }
    }
}
pub fn fetch_value_with_key<'a>(key: &Bytes, db: &'a mut Ferredis) -> Option<&'a DataStruct> {
    expire_if_needed(db, key);
    db.store.get(key).map(|entry| &entry.value)
}
pub fn fetch_entry_with_key<'a>(key: &Bytes, db: &'a mut Ferredis) -> Option<&'a mut Entry> {
    expire_if_needed(db, key);
    db.store.get_mut(key)
}

pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("before 1970").as_secs()
}
