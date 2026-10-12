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
impl Config {
    pub fn get_field(&self, key: &str) -> Option<String> {
        match key {
            "port" => Some(self.port.to_string()),
            "bind" => Some(self.bind.clone()),
            "maxmemory" => Some(self.maxmemory.to_string()),
            "maxmemory_policy" => Some(self.maxmemory_policy.clone()),
            "maxclients" => Some(self.maxclients.to_string()),
            "engine_channel_capacity" => Some(self.engine_channel_capacity.to_string()),
            "appendonly" => Some(self.appendonly.to_string()),
            "appendfilename" => Some(self.appendfilename.clone()),
            _ => None,
        }
    }
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), &'static str> {
        match key {
            "port" => {
                self.port = value.parse().map_err(|_| "Invalid Port value")?;
            }
            "bind" => {
                self.bind = value.to_string();
            }
            "maxmemory" => {
                self.maxmemory = value.parse().map_err(|_| "ERR Invalid maxmemory value")?;
            }
            "maxmemory_policy" => {
                self.maxmemory_policy = value.to_string();
            }
            "maxclients" => {
                self.maxmemory = value.parse().map_err(|_| "Err Invalid maxclients value")?;
            }
            "engine_channel_capacity" => {
                self.engine_channel_capacity = value
                    .parse()
                    .map_err(|_| "ERR Invalid capacity value")?;
            }
            "appendonly" => {
                self.appendonly = value
                    .parse()
                    .map_err(|_| "ERR Invalid appendonly value (expected true/false)")?;
            }
            "appendfilename" => {
                self.appendfilename = value.to_string();
            }
            _ => {
                return Err("ERR Unknown configuration parameter");
            }
        }
        Ok(())
    }
}
