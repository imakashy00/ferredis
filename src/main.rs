use tokio::net::{ TcpListener, TcpStream };
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::sync::Semaphore;
use std::io;
use std::sync::Arc;
use bytes::BytesMut;
use clap::Parser;

mod server;
mod protocol;

use crate::protocol::codec::{ decode, execute };
use crate::server::Config;
use crate::protocol::Frame;

async fn handle_stream(mut stream: TcpStream) -> io::Result<()> {
    // Dynamically Growable buffer
    let mut input_buffer = BytesMut::new(); // I can use Vec<u8> with read_to_end and take but is messay and complex for persistent loop(for upcoming bytes in the stream)
    let mut output_buffer = BytesMut::new(); // for sending output for commands

    // loop for persistence connection
    loop {
        let bytes = stream.read_buf(&mut input_buffer).await?; // exact count of valid bytes recieved
        if bytes == 0 {
            if !input_buffer.is_empty() {
                eprintln!("Warning: Client disconnected, left partial frame: {:?}", input_buffer);
            } else {
                eprintln!("Client Disconnected!");
            }
            return Ok(());
        }
        loop {
            match decode(&mut input_buffer) {
                Ok(Some(Frame::Array(Some(items)))) if items.is_empty() => {
                    continue;
                }
                // Execute the frame and store the encoded response in output buffer
                Ok(Some(frame)) => execute(frame).encode(&mut output_buffer),
                Ok(None) => {
                    break;
                } // no more bytes
                Err(e) => {
                    Frame::Error(format!("Protocol error: {e}")).encode(&mut output_buffer);
                    stream.write_all(&output_buffer).await?;
                    return Ok(());
                }
            }
        }

        // One write for all the inputs
        if !output_buffer.is_empty() {
            stream.write_all(&output_buffer).await?;
            output_buffer.clear();
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize config
    let config = Config::parse();
    let addr = format!("{}:{}", config.bind, config.port);

    // Bind Port
    let listener = TcpListener::bind(addr).await?;
    eprintln!("Server running on port:6379...");

    // Semaphore control how many tasks or threads can access a shared resource at the same time
    let semaphore = Arc::new(Semaphore::new(config.maxclients as usize));

    //accept in loop for new incoming connections
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                // Accquire permit without blocking the accept thread
                let sem_clone = Arc::clone(&semaphore);
                match sem_clone.try_acquire_owned() {
                    Ok(permit) => {
                        // spawn one task/bg task (green thread) per clinet
                        tokio::spawn(async move {
                            if let Err(e) = handle_stream(stream).await {
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
