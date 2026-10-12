use bytes::Bytes;

use crate::{ core::{ commands::registry::COMMAND_TABLE, keyspace::Ferredis }, protocol::Frame };

pub fn handle_ping(_db: &mut Ferredis, _args: &[Bytes]) -> Frame {
    Frame::Simple("PONG".into())
}
pub fn handle_echo(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    if args.len() == 2 {
        Frame::Bulk(Some(args[1].clone()))
    } else {
        Frame::Error("missing second argument".into())
    }
}
pub fn handle_commands(_db: &mut Ferredis, _args: &[Bytes]) -> Frame {
    Frame::Array(
        Some(
            COMMAND_TABLE.iter()
                .map(|command| Frame::Simple(command.name.to_string()))
                .collect()
        )
    )
}

pub fn handle_config_get(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Array(
        Some(
            args
                .iter()
                .skip(2)
                .map(|arg| String::from_utf8_lossy(arg).to_lowercase())
                .flat_map(|key| {
                    db.config
                        .get_field(&key)
                        .map(|value|
                            vec![Frame::Bulk(Some(key.into())), Frame::Bulk(Some(value.into()))]
                        )
                })
                .flatten()
                .collect()
        )
    )
}
pub fn handle_config_set(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    // lenth should be even so every key has a value
    if (args.len() & 1) != 0 {
        return Frame::Error("Value missing for key".into());
    }
    // map on each key value papir and set it
    let mut i = 2;
    while i < args.len() {
        let key = String::from_utf8_lossy(&args[i]).to_lowercase();
        let value = String::from_utf8_lossy(&args[i + 1]).to_lowercase();
        if let Err(error) = db.config.set_value(&key, &value) {
            return Frame::Error(format!("{}", error));
        }
        i += 2;
    }
    Frame::Simple("OK".into())
}

pub fn handle_config(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    let sub_cmd = args[1].to_ascii_uppercase();
    match sub_cmd.as_slice() {
        b"GET" => {
            if args.len() < 3 {
                Frame::Error(
                    format!(
                        "ERR Unknown arguments to config: {}",
                        String::from_utf8_lossy(&sub_cmd)
                    )
                )
            } else {
                handle_config_get(db, args)
            }
        }
        b"SET" => {
            if args.len() < 4 {
                Frame::Error(
                    format!("Missing arguments to config: {}", String::from_utf8_lossy(&sub_cmd))
                )
            } else {
                handle_config_set(db, args)
            }
        }
        _ =>
            Frame::Error(
                format!("ERR Unknown arguments to config: {}", String::from_utf8_lossy(&sub_cmd))
            ),
    }
}
pub fn handle_dbsize(db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple(db.current_memory_usage.to_string())
}
pub fn handle_flushall(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("FLUSHALL".into())
}

pub fn handle_save(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("SAVE".into())
}
pub fn handle_shutdown(_db: &mut Ferredis, args: &[Bytes]) -> Frame {
    Frame::Simple("SHUTDOWN".into())
    // lot of thing to do
    // 0. Stop accepting connection (may not be here )
    // 1. finish ongoing tasks,
    // 2. append everything to aof file
    // 3. Stop server
}
