// Holds the command table

use bytes::Bytes;

use crate::{
    core::{
        commands::{
            hashmap_cmd, keyspace_cmd, list_cmd, pub_sub_cmd, server_cmd, set_cmd, string_cmd,
            transactions_cmd,
        },
        keyspace::Ferredis,
    },
    protocol::Frame,
};
// handler type
type Handler = fn(&mut Ferredis, &[Bytes]) -> Frame;

#[derive(Debug, PartialEq, Eq)]
pub enum CommandFlag {
    Write,
    ReadOnly,
    DenyOom,
    Admin,
}

pub struct CommandMetadata {
    pub name: &'static str,
    pub arity: i32,
    pub flags: &'static [CommandFlag],
    pub first_key: i32,
    pub last_key: i32, // range boundary
    pub key_step: i32,
    pub handler: Handler,
}

pub static COMMAND_TABLE: &[CommandMetadata] = &[
    // Server
    CommandMetadata {
        name: "PING",
        arity: -1, // PING or PING "hello" (At least 1 arg)
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
        handler: server_cmd::handle_ping,
    },
    CommandMetadata {
        name: "ECHO",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
        handler: server_cmd::handle_echo,
    },
    CommandMetadata {
        name: "COMMANDS",
        arity: 1,
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
        handler: server_cmd::handle_commands,
    },
    CommandMetadata {
        name: "DBSIZE",
        arity: 1,
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
        handler: server_cmd::handle_dbsize,
    },
    CommandMetadata {
        name: "FLUSHDB",
        arity: 1, // Exact: FLUSHDB
        flags: &[CommandFlag::Write, CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: server_cmd::handle_flushdb,
    },
    CommandMetadata {
        name: "CONFIG",
        arity: -2,
        flags: &[CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: server_cmd::handle_config,
    },
    CommandMetadata {
        name: "SAVE",
        arity: 1, // Exact: SAVE
        flags: &[CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: server_cmd::handle_save,
    },
    CommandMetadata {
        name: "SHUTDOWN",
        arity: -1, // SHUTDOWN or SHUTDOWN SAVE / NOSAVE
        flags: &[CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: server_cmd::handle_shutdown,
    },
    // Keyspace
    CommandMetadata {
        name: "EXISTS",
        arity: -2, // EXISTS key1 key2 ... (At least 2 args)
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Every argument from index 1 to the end is a key!
        handler: keyspace_cmd::handle_exists,
    },
    CommandMetadata {
        name: "DEL",
        arity: -2, // DEL key1 key2 ... (At least 2 args)
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Mutates multiple keys
        handler: keyspace_cmd::handle_del,
    },
    CommandMetadata {
        name: "KEYS", // Returns all key names that match a pattern.
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Mutates multiple keys
        handler: keyspace_cmd::handle_keys,
    },
    CommandMetadata {
        name: "TTL",
        arity: 2, // Exact: TTL key
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: keyspace_cmd::handle_ttl,
    },
    CommandMetadata {
        name: "EXPIRE", // Set a timeout on key.
        arity: -3,      // EXPIRE key seconds [NX | XX | GT | LT] (At least 3 args)
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: keyspace_cmd::handle_expire,
    },
    CommandMetadata {
        name: "PERSIST",
        arity: 2, // Remove expiration time of a key
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: keyspace_cmd::handle_persist,
    },
    CommandMetadata {
        name: "RENAME",
        arity: 3, // Exact: RENAME old_key new_key
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 2,
        key_step: 1, // First key is at 1, last key is at 2!
        handler: keyspace_cmd::handle_rename,
    },
    // String
    CommandMetadata {
        name: "GET",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_get,
    },
    CommandMetadata {
        name: "SET",
        arity: -3, // Can have extra args like EX/NX
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_set,
    },
    CommandMetadata {
        name: "GETDEL",
        arity: 2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_get_del,
    },
    CommandMetadata {
        name: "MSET", // Multiple SET
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: -1,
        key_step: 2,
        handler: string_cmd::handle_mset,
    },
    CommandMetadata {
        name: "MGET",
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
        handler: string_cmd::handle_mget,
    },
    CommandMetadata {
        name: "INCR",
        arity: 2,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_incr,
    },
    CommandMetadata {
        name: "DECR",
        arity: 2,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_decr,
    },
    CommandMetadata {
        name: "INCRBY",
        arity: 3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_incrby,
    },
    CommandMetadata {
        name: "DECRBY",
        arity: 3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_decrby,
    },
    CommandMetadata {
        name: "STRLEN",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: string_cmd::handle_strlen,
    },
    // List (double-ended queue)
    CommandMetadata {
        name: "LPUSH",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lpush,
    },
    CommandMetadata {
        name: "RPUSH",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_rpush,
    },
    CommandMetadata {
        name: "LPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lpop,
    },
    CommandMetadata {
        name: "RPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_rpop,
    },
    CommandMetadata {
        name: "LRANGE",
        arity: 4,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lrange,
    },
    CommandMetadata {
        name: "LINDEX",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lindex,
    },
    CommandMetadata {
        name: "LSET", // set value at ind
        arity: 4,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lset,
    },
    CommandMetadata {
        name: "LREM", // remove value at ind
        arity: 4,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_lrem,
    },
    CommandMetadata {
        name: "LTRIM", // trims an existing list so that it contains only the specified range of elements
        arity: 4,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: list_cmd::handle_ltrim,
    },
    // Hashmap
    CommandMetadata {
        name: "HSET",
        arity: -4,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hset,
    },
    CommandMetadata {
        name: "HGET",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hget,
    },
    CommandMetadata {
        name: "HMGET",
        arity: -3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hmget,
    },
    CommandMetadata {
        name: "HEXISTS",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hexists,
    },
    CommandMetadata {
        name: "HLEN",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hlen,
    },
    CommandMetadata {
        name: "HGETALL",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hgetall,
    },
    CommandMetadata {
        name: "HKEYS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hkeys,
    },
    CommandMetadata {
        name: "HVALS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: hashmap_cmd::handle_hvals,
    },
    // Set
    CommandMetadata {
        name: "SADD",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_sadd,
    },
    CommandMetadata {
        name: "SREM",
        arity: -3,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_srem,
    },
    CommandMetadata {
        name: "SISMEMBER",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_sismember,
    },
    CommandMetadata {
        name: "SMEMBERS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_smembers,
    },
    CommandMetadata {
        name: "SCARD", // No of members in the set
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_scard,
    },
    CommandMetadata {
        name: "SPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_spop,
    },
    CommandMetadata {
        name: "SRANDMEMBER",
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
        handler: set_cmd::handle_srandmember,
    },
    CommandMetadata {
        name: "SUNION", // returns union of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
        handler: set_cmd::handle_sunion,
    },
    CommandMetadata {
        name: "SINTER", // returns the intersect of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
        handler: set_cmd::handle_sinter,
    },
    CommandMetadata {
        name: "SDIFF", // returns the diff of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
        handler: set_cmd::handle_sdiff,
    },
    // Transactions
    CommandMetadata {
        name: "MULTI",
        arity: 1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: transactions_cmd::handle_multi,
    },
    CommandMetadata {
        name: "EXEC",
        arity: 1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: transactions_cmd::handle_exec,
    },
    // Pub/Sub
    CommandMetadata {
        name: "PUBLISH",
        arity: 3,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: pub_sub_cmd::handle_publish,
    },
    CommandMetadata {
        name: "UNSUBSCRIBE",
        arity: -1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
        handler: pub_sub_cmd::handle_subscribe,
    },
];
