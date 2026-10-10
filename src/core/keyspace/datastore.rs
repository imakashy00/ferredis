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
}

impl Ferredis {
    pub fn new() -> Self {
        Ferredis { store: HashMap::new() }
    }
}
