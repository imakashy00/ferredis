// **[Binary file format]**
// 1. Magic Number and Version (4 bits): For file identification and file ready algorithm version
// 2. Database Selector: Tells the upcoming bytes belongs to which database
//     (OP Code with single byte followed by db index 0x00 for db 0)
// 3. Metadata:
//          i) Fererdis version,
//         ii) Ferredis bits arch 32 or 64,
//        iii) Creation time,
//         iv) Used-mem (memory required to put all in the file)
// 4. Data loop: ┌──────────────┬──────────────────┬─────────────┬──────────────┬─────────┬────────────────┬─────────┐
//               │ TTL Op-Code  │ Expiry Timestamp │  Data Type  │  Key Length  │   Key   │  Value Length  │  Value  │
//               │   (1 byte)   │    (8 bytes)     │  (1 byte)   │  (Variable)  │ (Bytes) │   (Variable)   │ (Bytes) │
//               └──────────────┴──────────────────┴─────────────┴──────────────┴─────────┴────────────────┴─────────┘

// 5. EOF Marker: A single byte op code outside the data
// 6. Checksum: two options for me
//              i) SHA256 (More secure but more memory overhead 32 bytes)
//             ii) CRC64 (Less secure but less memory overhead 8 bytes)

// --------------------------------------------------------------------------------------------------//
// **[Background Saving Process]**
// Triggers: Save Command or X command fulfilled per Y sec
// Fork the main process (syscall fork()):
//      Creates two process parent and child
//          Parent(main)-> returns to serve the clients
//          Child(saver)-> inherits point-in-time of the db memory copy via Copy on Write
// Child writes the db to a temp file to prevent data corruption of old snapshot
// Once file is finished and checksum is verified it reanames it to 'snap.rdb'
// Parent process thorugh sigchld process finds out child process is done to it frees the snapshot flag and the process is killed by OS

// --------------------------------------------------------------------------------------------------//

// **[Load the data into memory]**
// Locate and open file
// Verfy the magic number
// Read the file byte-by-byte
// Encounter the EOF and verify the checkSum

pub mod snapshot;

pub use snapshot::*;
