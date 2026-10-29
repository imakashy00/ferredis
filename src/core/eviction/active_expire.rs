use std::time::Duration;

use crate::core::{
    commands::{fetch_entry_with_key, now_secs},
    keyspace::Ferredis,
};
use bytes::Bytes;
use rand::seq::index;
// Run a infinite loop using tokio
pub fn run_active_expiry(db: &mut Ferredis, time_budget: Duration) {
    let start = now_secs();
    loop {
        let length = db.ttl_map.keys.len();
        let mut rng = rand::rng();
        let amount = length.min(20);
        if amount == 0 {
            break;
        }
        // get the sample vector
        let sample: Vec<Bytes> = index::sample(&mut rng, length, amount)
            .iter()
            .map(|i| db.ttl_map.keys[i].clone())
            .collect();
        // check which key expired; count it
        let mut expired = 0;
        for key in &sample {
            if let Some(entry) = fetch_entry_with_key(&key, db) {
                // remove the key from TTLMap

                if entry.expires_at <= Some(start) {
                    swap_and_remove(key, db);
                    expired += 1;
                }
            }
        }
        // less then 25% is expired
        if (expired * 100) / amount < 25 {
            break;
        }
        // if interval < start break
        if now_secs() - start >= time_budget.as_secs() {
            break;
        }
    }
}
fn swap_and_remove(key: &Bytes, db: &mut Ferredis) {
    // find the current keys position in position of the ttl map
    let Some(index) = db.ttl_map.positions.remove(key) else {
        panic!("Key not found in positions map")
    };
    // find the last index
    let last_ind = db.ttl_map.keys.len() - 1;
    // swap the current key with the last one and pop
    let key = db.ttl_map.keys.swap_remove(index);
    // update the index of key that moved from last
    if index < last_ind {
        let moved_key = &db.ttl_map.keys[index];
        db.ttl_map.positions.insert(moved_key.clone(), index);
    }
    // delete the entry form the db
    let Some(entry) = db.store.remove(&key) else {
        panic!("Key not found in store to delete!")
    };
    // restore memory
    db.current_memory_usage -= entry.approx_size;
}
