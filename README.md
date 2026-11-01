# Ferredis 🦀

**Ferredis** is a Redis-like in-memory key-value server built from scratch in Rust.

It uses Tokio for asynchronous networking, the RESP protocol for client communication, and a centralized single-threaded execution engine for managing database commands.

The project explores the internals of building a database server, including network protocols, concurrency, memory management, data structures, and persistence.

## Features

- **RESP protocol:** Frame parsing and encoding for client-server communication.
- **Async networking:** TCP server built using Tokio.
- **Centralized execution engine:** Single-threaded command execution with a bounded MPSC channel.
- **Multiple data structures:** Strings, Lists, Hashes, and Sets.
- **Command dispatch:** Command registry with arity validation, flags, and OOM checks.
- **Expiration:** Key expiration with lazy and active expiration mechanisms.
- **Snapshot persistence:** Save the in-memory database to disk and restore it during startup.
- **Memory management:** Configurable maximum memory and eviction policy options.
- **CLI configuration:** Runtime configuration through command-line arguments.

## Architecture

Ferredis separates network connection handling from database command execution.

- **Networking layer:** Handles client connections, reads incoming RESP frames, and writes responses.
- **Protocol layer:** Decodes incoming frames and encodes outgoing responses.
- **Command engine:** Processes database commands sequentially using a centralized execution model.
- **Core datastore:** Manages keys, values, data structures, expiration, and memory usage.
- **Persistence layer:** Handles snapshot creation and database restoration.

This design keeps database execution centralized while allowing multiple clients to connect asynchronously.

## Getting Started

### Prerequisites

- Rust (stable toolchain)
- Cargo

Install Rust using [rustup](https://rustup.rs/) if needed.

### Build locally

```bash
git clone https://github.com/imakashy00/ferredis.git
cd ferredis
cargo build
```

### Run the server

Start with the default configuration (`127.0.0.1:6379`):

```bash
cargo run
```

For an optimized build:

```bash
cargo run --release
```

Run with custom options:

```bash
cargo run --release -- \
  --port 6380 \
  --bind 127.0.0.1 \
  --maxmemory 104857600 \
  --maxclients 1000
```

See all available options:

```bash
cargo run -- --help
```

## Benchmarks

Ferredis was benchmarked against Redis using `redis-benchmark`.

### PING_MBULK

| Metric               | Ferredis |    Redis |
| -------------------- | -------: | -------: |
| Requests per second  |  220,264 |  234,192 |
| Median latency (p50) | 0.151 ms | 0.111 ms |
| Concurrent clients   |       50 |       50 |
| Total requests       |  100,000 |  100,000 |

<img width="669" height="158" alt="Screenshot 2026-11-01 at 9 21 00 PM" src="https://github.com/user-attachments/assets/1972af56-35a1-49d1-a55d-67cb6acab3ed" />
<img width="733" height="164" alt="Screenshot 2026-11-01 at 9 24 38 PM" src="https://github.com/user-attachments/assets/8cb7676c-10c7-4d32-9142-dce3e4cb020e" />
<img width="824" height="546" alt="Screenshot 2026-11-01 at 9 26 22 PM" src="https://github.com/user-attachments/assets/ca86b610-54d4-4095-88f0-35253f4884e1" />

**Ferredis achieved approximately 94.05% of Redis's throughput** in this particular benchmark.

### Reproduce the benchmark

Start Ferredis:

```bash
cargo run --release -- --port 6379
```

Run the benchmark:

```bash
redis-benchmark \
  -h 127.0.0.1 \
  -p 6379 \
  -t ping_mbulk \
  -n 100000 \
  -c 50 \
  -q
```

Run the same benchmark against Redis on port `6380`:

```bash
redis-benchmark \
  -h 127.0.0.1 \
  -p 6380 \
  -t ping_mbulk \
  -n 100000 \
  -c 50 \
  -q
```

**Benchmark notes:**

- `-n 100000`: Total number of requests.
- `-c 50`: Number of concurrent clients.
- `ping_mbulk`: Tests RESP-encoded PING commands.
- `-q`: Quiet output.

These are preliminary local benchmark results. The Ferredis result was collected from a development build, so rerun both benchmarks using the optimized Ferredis release build before treating the comparison as definitive.

PING is a lightweight operation and primarily measures protocol and request/response overhead. More benchmarks covering GET, SET, and other data structures are needed to evaluate overall database performance.

## Supported Commands

| Category | Supported Commands |
|---|---|
| **Server** | `PING`, `ECHO`, `COMMANDS`, `DBSIZE`, `FLUSHDB`, `CONFIG`, `SAVE` |
| **Keyspace** | `EXISTS`, `DEL`, `KEYS`, `TTL`, `EXPIRE`, `PERSIST`, `RENAME` |
| **Strings** | `GET`, `SET`, `GETDEL`, `MSET`, `MGET`, `INCR`, `DECR`, `INCRBY`, `DECRBY`, `STRLEN` |
| **Lists** | `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LRANGE`, `LINDEX`, `LSET`, `LREM`, `LTRIM` |
| **Hashes** | `HSET`, `HGET`, `HMGET`, `HEXISTS`, `HLEN`, `HGETALL`, `HKEYS`, `HVALS` |
| **Sets** | `SADD`, `SREM`, `SISMEMBER`, `SMEMBERS`, `SCARD`, `SPOP`, `SRANDMEMBER`, `SUNION`, `SINTER`, `SDIFF` |


## Persistence

Ferredis supports snapshot persistence.

The `SAVE` command writes the current in-memory database state to a snapshot file. The server can restore the database from an existing snapshot during startup.

The snapshot implementation is being developed as part of Ferredis's persistence subsystem.

## Work in Progress

Some Redis capabilities are still under development:

- **Transactions:** `MULTI` and `EXEC`.
- **Pub/Sub:** `PUBLISH`, `SUBSCRIBE`, and `UNSUBSCRIBE`.
- **Additional persistence and recovery improvements.**
- **Performance benchmarking and optimization.**

These features are not yet fully implemented.

## Verify the Server

Connect using `redis-cli`:

```bash
redis-cli -p 6379 PING
redis-cli -p 6379 SET name ferredis
redis-cli -p 6379 GET name
```

Or send a raw RESP command using `nc`:

```bash
printf '*1\r\n$4\r\nPING\r\n' | nc 127.0.0.1 6379
```

## Development Commands

Format the code:

```bash
cargo fmt
```

Run Clippy:

```bash
cargo clippy -- -D warnings
```

Run tests:

```bash
cargo test
```

## Project Status

Ferredis is an actively developed learning and systems-programming project.

The core data structures, command execution, asynchronous networking, and snapshot persistence are implemented. Transactions and Pub/Sub are still in progress.

The goal is to continue exploring database internals and understand the engineering decisions behind systems like Redis.
