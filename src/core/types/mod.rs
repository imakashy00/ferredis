pub mod string;
pub mod set;
pub mod list;
pub mod hash;

use bytes::Bytes;
pub use string::*;
pub use set::*;
pub use list::*;
pub use hash::*;

use crate::core::{ commands::keyspace_cmd::now_secs, keyspace::{ DataStruct, Entry, Ferredis } };

pub fn fetch_value_with_key(key: &Bytes, db: &Ferredis) -> Option<DataStruct> {
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
pub fn fetch_entry_with_key(key: &Bytes, db: &Ferredis) -> Option<Entry> {
    match db.store.get(key) {
        Some(entry) => {
            // if its already expired then return Nil
            if entry.expires_at < Some(now_secs()) {
                return None;
            } else {
                // Return value
                return Some(entry.clone());
            }
        }
        None => {
            // if not available return Nil
            return None;
        }
    }
}
// files has no idea what Redis or RESP is

// Method Name	Signature	What it does
// new	fn new() -> Self	Initializes an empty list wrapper around a VecDeque.
// push_left	fn push_left(&mut self, value: Vec<u8>)	Adds an element to the front of the queue.
// pop_left	fn pop_left(&mut self) -> Option<Vec<u8>>	Removes and returns the front element, if it exists.
// len	fn len(&self) -> usize	Returns the total count of elements currently in the list.
