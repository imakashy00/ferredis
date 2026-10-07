use tokio::net::{ TcpListener, TcpStream };
use tokio::io::{ AsyncReadExt, AsyncWriteExt };
use tokio::sync::Semaphore;
use std::io;
use std::sync::Arc;
use bytes::BytesMut;
use clap::Parser;

mod server;
mod protocol;

use crate::server::Config;

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
            }
            println!("Client Disconnected!");
            return Ok(());
        }

        // The upcoming bytes can be valid utf-8 or invalid text-chars
        // from_utf_lossy used smart pointer Cow<_,str> for handling this situation
        // If the byte is valid use Cow::borrowed(&str)-(Zero memory allocation cost) else Cow::owned(to_string)-(Needs to be fixed)
        // Cow is smart pointer that optimize memory usage by avoiding unnecessary memory allocations
        let command = String::from_utf8_lossy(&input_buffer[..bytes]); // convert till valid byte
        println!("Received: {:?}", command);
        input_buffer.clear(); // consume it so the next read starts fresh

        stream.write_all(b"+OK\r\n").await?;
    }
    // Ok(())
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
                eprintln!("Error: {:?}", error);
            }
        }
    }
}
