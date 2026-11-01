use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about = "A custom Redis-like server", long_about = None)]
pub struct Config {
    #[arg(long, default_value_t = Config::default().port)]
    pub port: u16,
    #[arg(long, default_value_t = Config::default().bind)]
    pub bind: String,
    #[arg(long, default_value_t = Config::default().maxmemory)]
    pub maxmemory: usize,
    #[arg(long, default_value_t = Config::default().maxmemory_policy)]
    pub maxmemory_policy: String,
    #[arg(long, default_value_t = Config::default().maxclients)]
    pub maxclients: u32, //The maximum number of clients connections
    #[arg(long, default_value_t = Config::default().engine_channel_capacity)]
    pub engine_channel_capacity: usize, // The maximum number of in-flight database commands that can wait in the pipe
    #[arg(long, default_value_t = Config::default().snapshot)]
    pub snapshot: bool,
    #[arg(long, default_value_os_t = Config::default().snap_dir)]
    // os_t uses into() whereas _t() uses .to_string()
    pub snap_dir: PathBuf, // to which dir save data
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
            snapshot: false,
            snap_dir: match dirs::home_dir() {
                Some(path) => path,
                None => PathBuf::from("ferredis"),
            },
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
            "snapshot" => Some(self.snapshot.to_string()),
            "snap_dir" => Some(self.snap_dir.to_string_lossy().to_string()),
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
                self.maxclients = value.parse().map_err(|_| "Err Invalid maxclients value")?;
            }
            "engine_channel_capacity" => {
                self.engine_channel_capacity =
                    value.parse().map_err(|_| "ERR Invalid capacity value")?;
            }
            "snapshot" => {
                self.snapshot = value
                    .parse()
                    .map_err(|_| "ERR Invalid appendonly value (expected true/false)")?;
            }
            "snap_dir" => {
                self.snap_dir = PathBuf::from(value);
            }
            _ => {
                return Err("ERR Unknown configuration parameter");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.port, 6379);
        assert_eq!(config.bind, "127.0.0.1");
        assert_eq!(config.maxmemory, 0);
        assert_eq!(config.maxmemory_policy, "noeviction");
        assert_eq!(config.maxclients, 10000);
        assert_eq!(config.engine_channel_capacity, 1024);
        assert_eq!(config.snapshot, false);
    }
    #[test]
    fn test_get_feild() {
        let mut config = Config::default();
        config.port = 8000;
        config.bind = "0.0.0.0".to_string();
        config.engine_channel_capacity = 512;
        config.snapshot = true;

        assert_eq!(config.get_field("port"), Some("8000".to_string()));
        assert_eq!(config.get_field("bind"), Some("0.0.0.0".to_string()));
        assert_eq!(
            config.get_field("engine_channel_capacity"),
            Some("512".to_string())
        );
        assert_eq!(config.get_field("snapshot"), Some("true".to_string()));
    }
    #[test]
    fn test_set_feild() {
        let mut config = Config::default();

        assert!(config.set_value("port", "9090").is_ok());
        assert_eq!(config.port, 9090);

        assert!(config.set_value("bind", "192.168.1.1").is_ok());
        assert_eq!(config.bind, "192.168.1.1".to_string());

        assert!(config.set_value("maxmemory", "512").is_ok());
        assert_eq!(config.maxmemory, 512);

        assert!(config.set_value("snap_dir", "/tmp/redis").is_ok());
        assert_eq!(config.snap_dir, PathBuf::from("/tmp/redis"));
    }
    #[test]
    fn test_set_value_errors() {
        let mut config = Config::default();

        assert!(config.set_value("port", "NAN").is_err());
        assert_eq!(config.port, 6379);

        assert!(config.set_value("maxmemory", "-100").is_err());
        assert_eq!(config.maxmemory, 0);

        assert_eq!(
            config.set_value("key", "value"),
            Err("ERR Unknown configuration parameter")
        );
    }
    #[test]
    fn test_config_cli_parsing() {
        let args = vec![
            "ferredis_bin",
            "--port",
            "8080",
            "--bind",
            "0.0.0.0",
            "--maxmemory",
            "10",
            "--maxclients",
            "5000",
            "--engine-channel-capacity", // clap turns snake_case to kebab-case in cli
            "512",
            "--snapshot",
        ];
        // try_parse_from is available bcz of the Parser trait
        let config = Config::try_parse_from(args).unwrap();

        assert_eq!(config.port, 8080);
        assert_eq!(config.bind, "0.0.0.0".to_string());
        assert_eq!(config.snapshot, true);
        assert_eq!(config.maxclients, 5000);
    }
}
