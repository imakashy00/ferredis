use bytes::Bytes;

use crate::{
    core::{
        commands::{ registry::{ COMMAND_TABLE, CommandFlag, CommandMetadata } },
        keyspace::Ferredis,
    },
    protocol::Frame,
};

pub fn dispatch(db: &mut Ferredis, frame: Frame) -> Frame {
    // Everything should be the array of bulk string
    let Some(args) = into_args(frame) else {
        return Frame::Error("ERR Protocol: expected array of bulk strings".into());
    };
    let cmd_name = args[0].to_ascii_uppercase();
    let metadata = match find_metadata(&cmd_name) {
        Some(meta) => meta,
        None => {
            return Frame::Error(format!("Unknown command: {}", String::from_utf8_lossy(&cmd_name)));
        }
    };
    // Arity check
    if !arity_ok(metadata.arity, args.len()) {
        return Frame::Error(
            format!("ERR wrong number of arguments for '{}' command", metadata.name.to_lowercase())
        );
    }
    // check for oom
    if db.is_out_of_memory() && metadata.flags.contains(&CommandFlag::DenyOom) {
        return Frame::Error("OOM: command not allowed when used memory > 'maxmemory'.".into());
    }

    (metadata.handler)(db, &args)
}

pub fn into_args(frame: Frame) -> Option<Vec<Bytes>> {
    match frame {
        Frame::Array(Some(items)) if !items.is_empty() =>
            items
                .into_iter()
                .map(|f| {
                    match f {
                        Frame::Bulk(Some(b)) => Some(b),
                        _ => None,
                    }
                })
                .collect(),
        _ => None,
    }
}
fn arity_ok(arity: i32, given: usize) -> bool {
    let given = given as i32;
    if arity >= 0 {
        given == arity
    } else {
        given >= -arity
    }
}
fn find_metadata(cmd_name: &[u8]) -> Option<&'static CommandMetadata> {
    COMMAND_TABLE.iter().find(|meta| meta.name.as_bytes() == cmd_name)
}
