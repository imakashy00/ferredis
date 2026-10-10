pub mod list;
pub mod hashmap;
pub mod set;
pub mod string;
pub mod keyspace;
pub mod server;

// Holds the command table
// Check in coming Frame Array
// check Arity and apply flags
// Routes request to correct module

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
    },
    CommandMetadata {
        name: "ECHO",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
    },
    CommandMetadata {
        name: "DBSIZE",
        arity: 1,
        flags: &[CommandFlag::ReadOnly],
        first_key: 0,
        last_key: 0,
        key_step: 0, // Server commands touch NO database keys!
    },
    CommandMetadata {
        name: "FLUSHALL",
        arity: 1, // Exact: FLUSHALL
        flags: &[CommandFlag::Write, CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "CONFIG GET",
        arity: -3,
        flags: &[CommandFlag::ReadOnly, CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "CONFIG SET",
        arity: -4,
        flags: &[CommandFlag::ReadOnly, CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "SAVE",
        arity: 1, // Exact: SAVE
        flags: &[CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "SHUTDOWN",
        arity: -1, // SHUTDOWN or SHUTDOWN SAVE / NOSAVE
        flags: &[CommandFlag::Admin],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    // Keyspace
    CommandMetadata {
        name: "EXISTS",
        arity: -2, // EXISTS key1 key2 ... (At least 2 args)
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Every argument from index 1 to the end is a key!
    },
    CommandMetadata {
        name: "DEL",
        arity: -2, // DEL key1 key2 ... (At least 2 args)
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Mutates multiple keys
    },
    CommandMetadata {
        name: "KEYS", // Returns all key names that match a pattern.
        arity: 2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: -1,
        key_step: 1, // Mutates multiple keys
    },
    CommandMetadata {
        name: "TTL",
        arity: 2, // Exact: TTL key
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "EXPIRE",
        arity: -3, // EXPIRE key seconds [NX | XX | GT | LT] (At least 3 args)
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "PERSIST",
        arity: 2, // Remove expiration time of a key
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "RENAME",
        arity: 3, // Exact: RENAME old_key new_key
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 2,
        key_step: 1, // First key is at 1, last key is at 2!
    },
    // String
    CommandMetadata {
        name: "GET",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SET",
        arity: -3, // Can have extra args like EX/NX
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "GETDEL",
        arity: 2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "MSET", // Multiple SET
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: -1,
        key_step: 2,
    },

    CommandMetadata {
        name: "MGET",
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
    },
    CommandMetadata {
        name: "INCR",
        arity: 2,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "DECR",
        arity: 2,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "INCRBY",
        arity: 3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "DECRBY",
        arity: 3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "STRLEN",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },

    // List (double-ended queue)
    CommandMetadata {
        name: "LPUSH",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "RPUSH",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "RPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LRANGE",
        arity: 4,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LINDEX",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LSET", // set value at ind
        arity: 4,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LREM", // remove value at ind
        arity: 4,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "LTRIM", // trims an existing list so that it contains only the specified range of elements
        arity: 4,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    // Hashmap
    CommandMetadata {
        name: "HSET",
        arity: -4,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HGET",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HMGET",
        arity: -3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HEXISTS",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HLEN",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HGETALL",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HKEYS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "HVALS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },

    // Set
    CommandMetadata {
        name: "SADD",
        arity: -3,
        flags: &[CommandFlag::Write, CommandFlag::DenyOom],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SREM",
        arity: -3,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SISMEMBER",
        arity: 3,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SMEMBERS",
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SCARD", // No of members in the set
        arity: 2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SPOP",
        arity: -2,
        flags: &[CommandFlag::Write],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SRANDMEMBER",
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: 1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SUNION", // returns union of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SINTER", // returns the intersect of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
    },
    CommandMetadata {
        name: "SDIFF", // returns the diff of multiple sets.
        arity: -2,
        flags: &[CommandFlag::ReadOnly],
        first_key: 1,
        last_key: -1,
        key_step: 1,
    },
    // Transactions
    CommandMetadata {
        name: "MULTI",
        arity: 1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "EXEC",
        arity: 1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    // Pub/Sub
    CommandMetadata {
        name: "PUBLISH",
        arity: 3,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
    CommandMetadata {
        name: "UNSUBSCRIBE",
        arity: -1,
        flags: &[],
        first_key: 0,
        last_key: 0,
        key_step: 0,
    },
];

// registry + one module per type parse(->)
// 2. In commands/list.rs (The Request Handler)

// This file parses the incoming RESP protocol frames, looks up the list in the database, runs the command, and formats the output client reply.
// Method Name	Signature	What it does
// parse_lpush	fn parse_lpush(args: Vec<Frame>) -> Result<(String, Vec<Vec<u8>>), Error>	Parse: Validates that the client passed a key name and at least one value. Converts RESP Frames into standard Rust types.
// execute_lpush	fn execute_lpush(db: &mut Db, key: String, values: Vec<Vec<u8>>) -> Frame	Execute: Finds or creates the list under key in the database keyspace, loops through the values calling types::List::push_left, and returns an integer Frame representing the new list size.

// Command Semantics.
// Validates if the exact command rules are respected (correct number of items, right data types, flags).
