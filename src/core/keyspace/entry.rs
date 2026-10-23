use std::{ collections::{ HashMap, HashSet, VecDeque } };

use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum DataStruct {
    String(Bytes),
    List(VecDeque<Bytes>),
    Hash(HashMap<Bytes, Bytes>),
    Set(HashSet<Bytes>),
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub value: DataStruct,
    //  If Instant now > expiration time, the key is dead.
    pub expires_at: Option<u64>,
    // coarse logic timer tick loop
    pub last_access: u64,
    pub approx_size: usize,
}

pub struct TTLMap {
    pub keys: Vec<Bytes>,
    pub positions: HashMap<Bytes, usize>,
}

impl TTLMap {
    pub fn new() -> Self {
        TTLMap { keys: Vec::new(), positions: HashMap::new() }
    }
}
