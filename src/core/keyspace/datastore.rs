use std::collections::HashMap;

use bytes::Bytes;

use crate::{ core::keyspace::Entry, server::Config };

pub struct Ferredis {
    pub store: HashMap<Bytes, Entry>,
    pub current_memory_usage: usize,
    pub config: Config,
}

impl Ferredis {
    pub fn new(config: Config) -> Self {
        Ferredis { store: HashMap::new(), current_memory_usage: 0, config }
    }
    pub fn is_out_of_memory(&self) -> bool {
        if self.config.maxmemory == 0 {
            return false; // 0 means unlimited memory!
        }
        self.current_memory_usage >= self.config.maxmemory
    }
}
