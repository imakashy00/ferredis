
use std::{fs::File, io::{BufWriter, ErrorKind, Write}, os::unix::fs, path::{Path, PathBuf}};
use std::io::{Result,Error};
use nix::sys::wait::{ WaitPidFlag, WaitStatus, waitpid };


use crate::{core::{keyspace::Ferredis, now_secs}, server::config};

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
            | Ok(WaitStatus::StillAlive)
            | Ok(WaitStatus::Stopped(_, _))
            | Ok(WaitStatus::Continued(_)) => {/* Proces still alive or suspended */}
            Err(_) => {
                // safe fallback to none
                db.bg_child_pid = None;
            }
        }
    }
}
fn write_and_hash(stream:&mut BufWriter<File>, check_sum:&mut crc64fast::Digest, data:&[u8])-> std::io::Result<()> {
    stream.write_all(data)?;
    check_sum.write(data);
    Ok(())
}
pub fn execute_binary_dump(db: &mut Ferredis) -> Result<()> {
    // Open a temporary buffer stream: "temp.rdb"
    let db_dir = PathBuf::from(&db.config.appenddir);
    let temp_path = db_dir.join("temp.rdb");
    let temp_file = File::create(&temp_path)?;
    let mut stream = BufWriter::new(temp_file);

    // Initialize crc64_fast::Digest for every byte we right
    // There are other options more secure than crc64 like sha256 and sha512 more secure than crc64 but
    // they are computationally heavier and because here we are not sharing the data over network so speed matters more than security
    let mut check_sum = crc64fast::Digest::new();

    // Write magic string and version ("FERREDIS0011") & update digest
    let header = b"FERREDIS0011";
    let _ = write_and_hash(&mut stream, &mut check_sum, header);
    // Write Database Selector Op-Code (e.g., 0xFE) followed by DB index
    let db_opcode:u8 = 0xFE;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[db_opcode]);
    let db_index:u8 = 0;
    let _ = write_and_hash(&mut stream, &mut check_sum, &[db_index]);

    // We can add the hashmap size to create a hashmap and we don't have to grow it; saving us some time but
    // there is a problem the snapshot may contain expired entries
    // We have three options:
    // 1. Lax loader: creates hashmap with capacity, loads everything in the memory and the active expiry job will take care of it
    // 2. Strict laoder: creates hashmap with capacity, loads only the unexpired keys in memory leaving some emty space in the hashmap
    // 3. Do not use with capcity and let hashmap grow, but
    //      the problem with this is that its slow as growing hashmap requires creating new memory in heap and coping of allready exiting data
    //      Loop over `db.store` and serialize valid entries
    // We will follow the Strict loader as some extra left space is not problem
    let keys = db.store.keys().len();
    let ttl_keys = db.ttl_map.keys.len();
    let _ = write_and_hash(&mut stream, &mut check_sum, &keys.to_be_bytes());

    // Loop over `db.store` and serialize valid entries
    for (key,entry) in db.store.iter(){
        if let Some(expire_at) = entry.expires_at{
            if expire_at < now_secs(){continue;}
        // has ttl write the ttlOpcode [0xFC]
        let _ = write_and_hash(&mut stream, &mut check_sum, &[0xFC]);
        // write the 8byte ttl timestamp
        let _ = write_and_hash(&mut stream, &mut check_sum, &expire_at.to_be_bytes());
        }
    }
    // 6. Append EOF marker (0xFF)
    // 7. Write the final digest.sum64() bytes to the tail
    // 8. Atomically rename "temp.rdb" to "dump.rdb"
    Ok(())
}
