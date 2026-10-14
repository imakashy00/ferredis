# ferredis

`ferredis` is a Redis-like in-memory key-value server written in Rust.
It uses the RESP protocol, Tokio async networking, and a centralized single-threaded command execution engine.

## Features

- RESP frame parsing/encoding
- Async TCP server (`tokio`)
- In-memory datastore with multiple data structures
- Command dispatch table with arity and OOM checks
- Configurable runtime options via CLI flags

## Project structure

- `src/main.rs` - server bootstrap and connection handling
- `src/server` - runtime config and engine plumbing
- `src/protocol` - RESP frame codec and dispatcher
- `src/core` - datastore and command handlers

## Prerequisites

- Rust (stable toolchain)
- Cargo

Install Rust from <https://rustup.rs/> if needed.

## Setup locally

```bash
git clone https://github.com/imakashy00/ferredis.git
cd ferredis
cargo build
```

## Run

Run with defaults (`127.0.0.1:6379`):

```bash
cargo run
```

Run with custom options:

```bash
cargo run -- --port 6380 --bind 0.0.0.0 --maxmemory 104857600 --maxclients 1000
```

### CLI options

```text
--port <PORT>                                        [default: 6379]
--bind <BIND>                                        [default: 127.0.0.1]
--maxmemory <MAXMEMORY>                              [default: 0]
--maxmemory-policy <MAXMEMORY_POLICY>                [default: noeviction]
--maxclients <MAXCLIENTS>                            [default: 10000]
--engine-channel-capacity <ENGINE_CHANNEL_CAPACITY>  [default: 1024]
--appendonly
--appendfilename <APPENDFILENAME>                    [default: appendfilename.aof]
```

## Verify the server quickly

If `redis-cli` is available:

```bash
redis-cli -p 6379 PING
redis-cli -p 6379 SET name ferredis
redis-cli -p 6379 GET name
```

Or test with raw RESP over `nc`:

```bash
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 6379
```

## Supported commands (current status)

The server command registry currently includes:

### Server

- `PING`, `ECHO`, `COMMANDS`, `DBSIZE`, `FLUSHDB`, `CONFIG`
- `SAVE`, `SHUTDOWN` are registered but not implemented yet (currently TODO)

### Keyspace

- `EXISTS`, `DEL`, `KEYS`, `TTL`, `EXPIRE`, `PERSIST`, `RENAME`

### Strings

- `GET`, `SET`, `GETDEL`, `MSET`, `MGET`, `INCR`, `DECR`, `INCRBY`, `DECRBY`, `STRLEN`

### Lists

- `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE`, `LINDEX`, `LSET`, `LREM`, `LTRIM`

### Hashes

- `HSET`, `HGET`, `HMGET`, `HEXISTS`, `HLEN`, `HGETALL`, `HKEYS`, `HVALS`

### Sets

- `SADD`, `SREM`, `SISMEMBER`, `SMEMBERS`, `SCARD`, `SPOP`, `SRANDMEMBER`, `SUNION`, `SINTER`, `SDIFF`

### In progress / placeholders

- Transaction and Pub/Sub handlers are present (`MULTI`, `EXEC`, `PUBLISH`, `UNSUBSCRIBE`) but currently act as placeholders.

## Developer commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

## Notes

- This project is actively evolving and not yet feature-complete.
- Some command families are implemented fully, while others are scaffolded for upcoming work.
