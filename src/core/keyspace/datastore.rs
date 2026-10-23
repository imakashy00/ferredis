use std::collections::HashMap;

use bytes::Bytes;

use crate::{ core::{ commands::now_secs, keyspace::{ Entry, TTLMap } }, server::Config };

pub struct Ferredis {
    pub store: HashMap<Bytes, Entry>,
    pub ttl_map: TTLMap,
    pub current_memory_usage: usize,
    pub config: Config,
}

impl Ferredis {
    pub fn new(config: Config) -> Self {
        Ferredis {
            store: HashMap::new(),
            ttl_map: TTLMap::new(),
            current_memory_usage: 0,
            config,
        }
    }
    pub fn is_out_of_memory(&self) -> bool {
        if self.config.maxmemory == 0 {
            return false; // 0 means unlimited memory!
        }
        self.current_memory_usage >= self.config.maxmemory
    }
    pub fn set_string_keep_ttl(&mut self, key: Bytes, value: Bytes) {
        let size_kv = 32 + key.len() + value.len();
        match self.store.get_mut(&key) {
            Some(entry) => {
                self.current_memory_usage =
                    self.current_memory_usage.saturating_sub(entry.approx_size) + size_kv;
                value.len();
                entry.value = super::DataStruct::String(value);
                entry.approx_size = size_kv;
                entry.last_access = now_secs();
            }
            None => {
                self.current_memory_usage += size_kv;
                self.store.insert(key, Entry {
                    value: super::DataStruct::String(value),
                    expires_at: None,
                    last_access: now_secs(),
                    approx_size: size_kv,
                });
            }
        }
    }
}
