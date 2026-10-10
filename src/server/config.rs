use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "A custom Redis-like server", long_about = None)]
pub struct Config {
    #[arg(long, default_value_t = 6379)]
    pub port: u16,
    #[arg(long, default_value_t = String::from("127.0.0.1".to_string()))]
    pub bind: String,
    #[arg(long, default_value_t = 0)]
    pub maxmemory: usize,
    #[arg(long, default_value_t = String::from("noeviction".to_string()))]
    pub maxmemory_policy: String,
    #[arg(long, default_value_t = 10000)]
    pub maxclients: u32, //The maximum number of clients connections
    #[arg(long, default_value_t = 1024)]
    pub engine_channel_capacity: usize, // The maximum number of in-flight database commands that can wait in the pipe
    #[arg(long, default_value_t = false)]
    pub appendonly: bool,
    #[arg(long, default_value_t = String::from("appendfilename.aof".to_string()))]
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
