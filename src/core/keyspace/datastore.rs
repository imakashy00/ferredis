use std::collections::HashMap;

use bytes::Bytes;

use crate::core::keyspace::Entry;

pub struct Ferredis {
    pub store: HashMap<Bytes, Entry>,
    pub current_memory_usage: usize,
    pub maxmemory: usize,
}

impl Ferredis {
    pub fn new(maxmemory: usize) -> Self {
        Ferredis { store: HashMap::new(), current_memory_usage: 0, maxmemory }
    }
    pub fn is_out_of_memory(&self) -> bool {
        if self.maxmemory == 0 {
            return false; // 0 means unlimited memory!
        }
        self.current_memory_usage >= self.maxmemory
    }
}
