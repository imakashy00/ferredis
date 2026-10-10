use tokio::net::TcpListener;
use tokio::io::AsyncWriteExt;
use tokio::sync::{ Semaphore, mpsc };
use std::io;
use std::sync::Arc;
use clap::Parser;

mod server;
mod protocol;
mod core;

use crate::core::keyspace::Ferredis;
use crate::protocol::codec::dispatch;
use crate::server::{ CommandMsg, Config, handle_stream };

#[tokio::main]
async fn main() -> io::Result<()> {
    // TODO:: Graceful shutdown
    // Initialize config
    let config = Config::parse();
    let addr = format!("{}:{}", config.bind, config.port);

    // Bind Port
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Server running on port:6379...");

    // Semaphore control how many tasks or threads can access a shared resource at the same time
    let semaphore = Arc::new(Semaphore::new(config.maxclients as usize));
    // Configure bounded channels
    let (engine_tx, mut engine_rx) = mpsc::channel::<CommandMsg>(
        config.engine_channel_capacity as usize
    );
    // Spawn the centralized Redis execution engine (Single-threaded execution state)
    tokio::spawn(async move {
        // Single threaded so no Arc<Mutex<>> required here 💪
        let mut db = Ferredis::new(config.maxmemory);
        while let Some(msg) = engine_rx.recv().await {
            let response_frame = dispatch(&mut db, msg.frame);
            // Send the result back to the specific client connection task
            let _ = msg.respond_to.send(response_frame);
        }
    });

    //accept in loop for new incoming connections
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                // Enable TCP_NODELAY
                // By default it is enabled to optimize bandwith
                // Nagle Algo instructs Kernal to hold small packets of data and release only is reicieves (Acknowledgement)ACK or Maximum Segment Size(MSS) is reached
                // for small data like +OK\r\n it becomes deadlock as kernal waits for ACK and client wait for response to send ACK
                // When disabled Kernal transmit every samll packet without and confirmation or waiting - prioritizing low latency over packet size.
                if let Err(e) = stream.set_nodelay(true) {
                    eprintln!("Failed to set TCP_NODELAY: {:?}", e);
                    continue; // Or handle error based on your strictness
                }
                // TODO::ENABLE TCP KEEPALIVE:

                // Accquire permit without blocking the accept thread
                let sem_clone = Arc::clone(&semaphore);
                // Clone the Send Channel
                let tx_clone = engine_tx.clone();
                match sem_clone.try_acquire_owned() {
                    Ok(permit) => {
                        // spawn one task/bg task (green thread) per clinet
                        tokio::spawn(async move {
                            if let Err(e) = handle_stream(stream, tx_clone).await {
                                eprintln!("Error handling stream: {:?}", e);
                            }
                            drop(permit); // drop to free the slot
                        });
                    }
                    Err(_) => {
                        println!("Rejected connection: maxclients limit reached.");
                        // stream drops here, closing the connection immediately}
                        let _ = stream.write_all(b"-ERR max number of clients reached\r\n").await;
                    }
                }
            }
            // failed to connect
            Err(error) => {
                eprintln!("Error in connection: {:?}", error);
            }
        }
    }
}
