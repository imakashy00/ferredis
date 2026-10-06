#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub bind: String,
    pub maxmemory: u64,
    pub maxmemory_policy: String,
    pub maxclients: u32, //The maximum number of physical TCP connections
    pub engine_channel_capacity: usize, // The maximum number of in-flight database commands that can wait in the pipe
    pub appendonly: bool,
    pub appendfilename: String, // to which file save data
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 6379,
            bind: "127.0.0.1".to_string(),
            maxmemory: 0, // unlimited
            maxmemory_policy: "noeviction".to_string(),
            maxclients: 10000,
            engine_channel_capacity: 1024,
            appendonly: false,
            appendfilename: "appendfilename.aof".to_string(),
        }
    }
}

