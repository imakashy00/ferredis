use nix::sys::wait::{ WaitPidFlag, WaitStatus, waitpid };

use crate::core::keyspace::Ferredis;

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

pub fn execute_binary_dump(_db: &mut Ferredis) -> std::io::Result<()> {
    // 1. Open a temporary buffer stream: "temp.rdb"
    // 2. Initialize crc64_fast::Digest
    // 3. Write magic string and version ("FERREDIS0011") & update digest
    // 4. Write Database Selector Op-Code (e.g., 0xFE) followed by DB index
    // 5. Loop over `db.store` and serialize valid entries
    // 6. Append EOF marker (0xFF)
    // 7. Write the final digest.sum64() bytes to the tail
    // 8. Atomically rename "temp.rdb" to "dump.rdb"
    Ok(())
}
