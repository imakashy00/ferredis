use bytes::Bytes;

use crate::{ core::{ commands::keyspace_cmd::now_secs, keyspace::{ DataStruct, Ferredis } } };

pub fn fetch_with_key(key: &Bytes, db: &mut Ferredis) -> Option<DataStruct> {
    match db.store.get(key) {
        Some(entry) => {
            // if its already expired then return Nil
            if entry.expires_at < Some(now_secs()) {
                return None;
            } else {
                // Return value
                return Some(entry.value.clone());
            }
        }
        None => {
            // if not available return Nil
            return None;
        }
    }
}
