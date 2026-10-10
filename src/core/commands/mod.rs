pub mod list;
pub mod hashmap;
pub mod set;
pub mod string;

// Holds the command table
// Check in coming Frame Array
// check Arity and apply flags
// Routes request to correct module

pub enum CommandFlag {
    Write,
    ReadOnly,
    DenyOom,
    Admin,
}

pub struct CommandMetadata {
    name: &'static str,
    arity: i32,
    flags: &'static [CommandFlag],
    first_key: i32,
    last_key: i32, // range boundary
    key_step: i32,
}

pub static COMMAND_TABLE: &[CommandMetadata] = &[
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
    
    // Set
];

// registry + one module per type parse(->)
// 2. In commands/list.rs (The Request Handler)

// This file parses the incoming RESP protocol frames, looks up the list in the database, runs the command, and formats the output client reply.
// Method Name	Signature	What it does
// parse_lpush	fn parse_lpush(args: Vec<Frame>) -> Result<(String, Vec<Vec<u8>>), Error>	Parse: Validates that the client passed a key name and at least one value. Converts RESP Frames into standard Rust types.
// execute_lpush	fn execute_lpush(db: &mut Db, key: String, values: Vec<Vec<u8>>) -> Frame	Execute: Finds or creates the list under key in the database keyspace, loops through the values calling types::List::push_left, and returns an integer Frame representing the new list size.

// Command Semantics.
// Validates if the exact command rules are respected (correct number of items, right data types, flags).
