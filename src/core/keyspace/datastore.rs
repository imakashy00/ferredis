use std::{ collections::{ HashMap, HashSet, VecDeque } };

use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum DataStruct {
    String(Bytes),
    List(VecDeque<Bytes>),
    Hash(HashMap<Bytes, Bytes>),
    Set(HashSet<Bytes>),
}
pub struct Entry {
    pub value: DataStruct,
    //  If Instant now > expiration time, the key is dead.
    pub expires_at: Option<u64>,
    // coarse logic timer tick loop
    pub last_access: u32,
    pub approx_size: usize,
}

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
