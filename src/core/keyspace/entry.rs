use std::{ collections::{ HashMap, HashSet, VecDeque } };

use bytes::Bytes;

#[derive(Debug, Clone)]
pub enum DataStruct {
    String(Bytes),
    List(VecDeque<Bytes>),
    Hash(HashMap<Bytes, Bytes>),
    Set(HashSet<Bytes>),
}
pub enum DataType {
    String,
    List,
    Hash,
    Set,
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
