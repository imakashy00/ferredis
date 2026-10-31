use bytes::Bytes;
use nix::{
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
    unistd::{ForkResult, fork},
};
use std::io::{BufReader, Error, ErrorKind, Read, Result};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::core::{DataStruct, Entry, keyspace::Ferredis, now_secs};

pub fn handle_snapshot(db: &mut Ferredis) {
    if let Some(child_pid) = db.bg_child_pid {
        let pid = nix::unistd::Pid::from(child_pid);

        // Os query
        match waitpid(pid, Some(WaitPidFlag::WNOHANG)) {
            // Non blcoking query call
            Ok(WaitStatus::Exited(_pid, _exit_code)) => {
                // reset the state
                db.bg_child_pid = None;
            }
            Ok(WaitStatus::Signaled(_pid, _signal, _core_dump)) => {
                eprintln!("Could not finish the snapshot job");
                db.bg_child_pid = None;
            }
            Ok(WaitStatus::StillAlive)
            | Ok(WaitStatus::Stopped(_, _))
            | Ok(WaitStatus::Continued(_)) => { /* Proces still alive or suspended */ }
            Err(_) => {
                // safe fallback to none
                db.bg_child_pid = None;
            }
        }
    } else {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                // parent continue process
                db.bg_child_pid = Some(child);
            }
            Ok(ForkResult::Child) => {
                // snapshot entire db
                if let Err(err) = execute_binary_dump(db) {
                    eprintln!("Failed to dump the db:{}", err);
                    std::process::exit(1);
                }
                // end the process
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Fork failed: {}", e);
            }
        }
    }
}
fn write_and_hash(
    stream: &mut BufWriter<File>,
    check_sum: &mut crc64fast::Digest,
    data: &[u8],
) -> std::io::Result<()> {
    stream.write_all(data)?;
    check_sum.write(data);
    Ok(())
}
pub fn execute_binary_dump(db: &mut Ferredis) -> Result<()> {
    // Open a temporary buffer stream: "temp.rdb"
    let db_dir = PathBuf::from(&db.config.snap_dir);
    let temp_path = db_dir.join("temp.rdb");
    let temp_file = File::create(&temp_path)?;
    let mut stream: BufWriter<File> = BufWriter::new(temp_file);

    // Initialize crc64_fast::Digest for every byte we right
    // There are other options more secure than crc64 like sha256 and sha512 more secure than crc64 but
    // they are computationally heavier and because here we are not sharing the data over network so speed matters more than security
    let mut check_sum = crc64fast::Digest::new();

    // Write magic string and version ("FERREDIS0011") & update digest
    let header = b"FERREDIS0011";
    let _ = write_and_hash(&mut stream, &mut check_sum, header);
    // 3. Metadata:
    //          i) Fererdis version,
    //         ii) Ferredis bits arch 32 or 64,
    //        iii) Creation time,
    //         iv) Used-mem (memory required to put all in the file)
    let meta_opcode = 0xFA;
    // i) Fererdis version,
    let _ = write_and_hash(&mut stream, &mut check_sum, &[meta_opcode]);
    let _ = write_and_hash(
        &mut stream,
        &mut check_sum,
        &"ferredis-ver".len().to_be_bytes(),
    );
    let _ = write_and_hash(&mut stream, &mut check_sum, b"ferredis-ver");
    let _ = write_and_hash(&mut stream, &mut check_sum, &"1.0.0".len().to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, b"1.0.0");
    // ii) Ferredis bits arch 32 or 64,
    let arch_bits = (std::mem::size_of::<usize>() * 8).to_string(); // Detects 64 or 32 dynamically
    let _ = write_and_hash(&mut stream, &mut check_sum, &[meta_opcode]);
    let _ = write_and_hash(
        &mut stream,
        &mut check_sum,
        &"ferredis-bits".len().to_be_bytes(),
    );
    let _ = write_and_hash(&mut stream, &mut check_sum, b"ferredis-bits");
    let _ = write_and_hash(&mut stream, &mut check_sum, &arch_bits.len().to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, arch_bits.as_bytes());
    // iii) Creation time
    let curr_time = now_secs();
    let _ = write_and_hash(&mut stream, &mut check_sum, &[meta_opcode]);
    let _ = write_and_hash(&mut stream, &mut check_sum, &"ctime".len().to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, b"ctime");
    let _ = write_and_hash(&mut stream, &mut check_sum, &curr_time.to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, &curr_time.to_be_bytes());
    // iv) Used memory
    let used_mem = db.current_memory_usage;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[meta_opcode]);
    let _ = write_and_hash(&mut stream, &mut check_sum, &"used_mem".len().to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, b"used_mem");
    let _ = write_and_hash(&mut stream, &mut check_sum, &used_mem.to_be_bytes());
    let _ = write_and_hash(&mut stream, &mut check_sum, &used_mem.to_be_bytes());

    // Write Database Selector Op-Code (e.g., 0xFE) followed by DB index
    let db_opcode: u8 = 0xFE;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[db_opcode]);
    let db_index: u8 = 0;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[db_index]);

    // We can add the hashmap size to create a hashmap and we don't have to grow it; saving us some time but
    // there is a problem the snapshot may contain expired entries
    // We have three options:
    // 1. Lax loader: creates hashmap with capacity, loads everything in the memory and the active expiry job will take care of it
    // 2. Strict laoder: creates hashmap with capacity, loads only the unexpired keys in memory leaving some emty space in the hashmap
    // 3. Do not use 'with capcity' and let hashmap grow, but
    //      the problem with this is that its slow as growing hashmap requires creating new memory in heap and coping of allready exiting data
    //      Loop over `db.store` and serialize valid entries
    // We will follow the Strict loader as some extra left space is not problem
    let keys = db.store.keys().len();
    let _ = write_and_hash(&mut stream, &mut check_sum, &keys.to_be_bytes());

    // Data loop: ┌──────────────┬──────────────────┬─────────────┬──────────────┬─────────┬────────────────┬─────────┐
    //            │ TTL Op-Code  │ Expiry Timestamp │  Data Type  │  Key Length  │   Key   │  Value Length  │  Value  │
    //            │   (1 byte)   │    (8 bytes)     │  (1 byte)   │  (Variable)  │ (Bytes) │   (Variable)   │ (Bytes) │
    //            └──────────────┴──────────────────┴─────────────┴──────────────┴─────────┴────────────────┴─────────┘

    // Loop over `db.store` and serialize valid entries
    for (key, entry) in db.store.iter() {
        if let Some(expire_at) = entry.expires_at {
            if expire_at < now_secs() {
                continue;
            } // expired entry
            // has ttl write the ttlOpcode [0xFC]
            let _ = write_and_hash(&mut stream, &mut check_sum, &[0xFC]);
            // write the 8byte ttl timestamp
            let _ = write_and_hash(&mut stream, &mut check_sum, &expire_at.to_be_bytes());
        }
        // data type
        let data_type: u8 = match &entry.value {
            DataStruct::String(_) => 0x00,
            DataStruct::List(_) => 0x01,
            DataStruct::Hash(_) => 0x10,
            DataStruct::Set(_) => 0x11,
        };
        let _ = write_and_hash(&mut stream, &mut check_sum, &[data_type]);
        // Key
        let key_len = key.len().to_be_bytes();
        let _ = write_and_hash(&mut stream, &mut check_sum, &key_len);
        let _ = write_and_hash(&mut stream, &mut check_sum, key);
        // Value
        match &entry.value {
            DataStruct::String(bytes) => {
                let str_val = bytes.len().to_be_bytes();
                let _ = write_and_hash(&mut stream, &mut check_sum, &str_val);
                let _ = write_and_hash(&mut stream, &mut check_sum, &bytes);
            }
            DataStruct::List(dq) => {
                let list_len = dq.len().to_be_bytes();
                let _ = write_and_hash(&mut stream, &mut check_sum, &list_len);
                for item in dq {
                    let _ = write_and_hash(&mut stream, &mut check_sum, item);
                }
            }
            DataStruct::Hash(hmap) => {
                let hash_len = hmap.len().to_be_bytes();
                let _ = write_and_hash(&mut stream, &mut check_sum, &hash_len);
                for (field, value) in hmap {
                    // write field
                    let field_len = field.len().to_be_bytes();
                    let _ = write_and_hash(&mut stream, &mut check_sum, &field_len);
                    let _ = write_and_hash(&mut stream, &mut check_sum, field);
                    // write val
                    let val_len = value.len().to_be_bytes();
                    let _ = write_and_hash(&mut stream, &mut check_sum, &val_len);
                    let _ = write_and_hash(&mut stream, &mut check_sum, value);
                }
            }
            DataStruct::Set(hset) => {
                let set_len = hset.len().to_be_bytes();
                let _ = write_and_hash(&mut stream, &mut check_sum, &set_len);
                for item in hset {
                    let _ = write_and_hash(&mut stream, &mut check_sum, item);
                }
            }
        }
    }
    // Append EOF marker (0xFF)
    let eof_marker: u8 = 0xFF;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[eof_marker]);
    // Write the final digest.sum64() bytes to the tail
    let digest_sum = check_sum.sum64().to_be_bytes();
    let _ = write_and_hash(&mut stream, &mut check_sum, &digest_sum);
    // flush the stream to disk
    stream.flush()?;
    // Atomically rename "temp.rdb" to "dump.rdb"
    std::fs::rename(&temp_path, "dump.rdb")?;
    Ok(())
}

fn read_bytes(stream: &mut BufReader<File>, check_sum: &mut crc64fast::Digest) -> Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes)?;
    check_sum.write(&len_bytes);
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    check_sum.write(&buf);

    Ok(buf)
}

fn read_string(stream: &mut BufReader<File>, check_sum: &mut crc64fast::Digest) -> Result<String> {
    let buf = read_bytes(stream, check_sum)?;
    String::from_utf8(buf).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

fn read_u64(stream: &mut BufReader<File>, check_sum: &mut crc64fast::Digest) -> Result<u64> {
    let mut bytes = [0u8; 8];
    stream.read_exact(&mut bytes)?;
    check_sum.write(&bytes);
    Ok(u64::from_be_bytes(bytes))
}

pub fn parse_snapshot_file(file: PathBuf, db: &mut Ferredis) -> Result<()> {
    // Open the file
    let snapshot_file = File::open(&file)?;
    let mut stream = BufReader::new(snapshot_file);
    // Initialize the check sum
    let mut check_sum = crc64fast::Digest::new();

    // validate header and version
    let mut header = [0u8; 12];
    stream.read_exact(&mut header)?;
    check_sum.write(&header);

    if &header != b"FERREDIS0011" {
        eprintln!("Invalid file format");
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Invalid file format: Magic header mismatch",
        ));
    }
    // Read file
    // 0xFA [metadata]
    // 0xFE Db index
    // 0xFC Timestamp
    // 0xFF EOF

    let mut single_byte = [0u8; 1];
    loop {
        // read the data and check sum it
        stream.read_exact(&mut single_byte)?;
        check_sum.write(&single_byte);
        let opcode = single_byte[0];
        match opcode {
            0xFA => {
                let meta_key = read_string(&mut stream, &mut check_sum)?;
                // metadata
                match meta_key.as_str() {
                    "ferredis-ver" => {
                        let version = read_string(&mut stream, &mut check_sum)?;
                        println!("Snapshot System Version: {}", version);
                    }
                    "ferredis-bits" => {
                        let bits = read_string(&mut stream, &mut check_sum)?;
                        let version = read_string(&mut stream, &mut check_sum)?;
                        println!("Snapshot System Version: {}", version);
                        // check arch type of the system
                        if bits != (std::mem::size_of::<usize>() * 8).to_string() {
                            println!("Warning: Architecture mismatch between file and host.");
                        }
                    }
                    "ctime" => {
                        let creation_time = read_u64(&mut stream, &mut check_sum)?;
                        println!("Snapshot Created At (Timestamp): {}", creation_time);
                    }
                    "used_mem" => {
                        let memory_used = read_u64(&mut stream, &mut check_sum)?;
                        println!("Database Memory Usage at save: {} bytes", memory_used);
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unknown metadata key: {}", meta_key),
                        ));
                    }
                }
            }
            0xFE => {
                // Db index
                // It tell which database the upcoming key value pair belongs to
                // right now we have only one db
                let mut index_buf = [0u8; 1];
                stream.read_exact(&mut index_buf)?;
                check_sum.write(&index_buf);

                let curr_db_ind = index_buf[0];
                println!(
                    "Switching parser context to Database Index: {}",
                    curr_db_ind
                );
                // for now only 0 ind
                if curr_db_ind != 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Unknown db index key: {}", curr_db_ind),
                    ));
                }
                // if ever support more then one db instance then it can be array of store or hashmap
                // - switch to that db and load the key value pair to it after parsing the key value pair
                // this all should be another 'loop'
            }
            0xFC => {
                // key values
                let mut expire_buf = [0u8; 8];
                stream.read_exact(&mut expire_buf)?;
                check_sum.write(&expire_buf);
                let expire_at_time = u64::from_be_bytes(expire_buf);

                let now = now_secs();
                let is_expired = expire_at_time < now;

                let mut type_buf = [0u8; 1];
                stream.read_exact(&mut type_buf)?;
                check_sum.write(&type_buf);
                // read the bytes
                let type_opcode = type_buf[0];
                let key_bytes = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                let value_struct = match type_opcode {
                    0x00 => {
                        let value_bytes = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                        DataStruct::String(value_bytes)
                    }
                    0x01 => {
                        let count = read_u64(&mut stream, &mut check_sum)?;
                        let mut deque = std::collections::VecDeque::with_capacity(count as usize);
                        for _ in 0..count {
                            let element = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                            deque.push_back(element);
                        }
                        DataStruct::List(deque)
                    }
                    0x02 => {
                        let count = read_u64(&mut stream, &mut check_sum)?;
                        let mut map = std::collections::HashMap::with_capacity(count as usize);
                        for _ in 0..count {
                            let field = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                            let val = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                            map.insert(field, val);
                        }
                        DataStruct::Hash(map)
                    }
                    0x04 => {
                        let count = read_u64(&mut stream, &mut check_sum)?;
                        let mut set = std::collections::HashSet::with_capacity(count as usize);
                        for _ in 0..count {
                            let element = Bytes::from(read_bytes(&mut stream, &mut check_sum)?);
                            set.insert(element);
                        }
                        DataStruct::Set(set)
                    }
                    _ => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Unknown type opcode: 0x{:02X}", type_opcode),
                        ));
                    }
                };
                if !is_expired {
                    let entry = Entry {
                        value: value_struct,
                        expires_at: Some(expire_at_time),
                        last_access: now,
                        approx_size: 0, // TODO: calculate approx size
                    };
                    // store in db
                    db.store.insert(key_bytes.clone(), entry);
                    // build ttl map
                    db.ttl_map.keys.push(key_bytes.clone());
                    db.ttl_map
                        .positions
                        .insert(key_bytes, db.ttl_map.keys.len() - 1);
                }
            }
            0xFF => {
                // EOF
                break;
            }
            _ => {
                // unexpected byte throw error
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unknown byte key: {:?}", single_byte),
                ));
            }
        }
    }
    // check sum
    let mut expected_checksum_buf = [0u8; 8];
    stream.read_exact(&mut expected_checksum_buf)?;
    let expected_checksum = u64::from_be_bytes(expected_checksum_buf);
    let calculated_checksum = check_sum.sum64();

    // validate the check sum

    if calculated_checksum != expected_checksum {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "CRITICAL DATA CORRUPTION DETECTED: Checksum mismatch😱",
        ));
    }
    println!("Snapshot loaded safely ...");
    Ok(())
}
