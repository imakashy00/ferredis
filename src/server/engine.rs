use std::io;

use bytes::BytesMut;
use tokio::{ io::{ AsyncReadExt, AsyncWriteExt }, net::TcpStream, sync::{ mpsc, oneshot } };

use crate::protocol::{ Frame, codec::decode };

pub struct CommandMsg {
    pub frame: Frame,
    // Oneshot channel allows the engine to send the reply back directly to specific client task
    pub respond_to: oneshot::Sender<Frame>,
}

pub async fn handle_stream(mut stream: TcpStream, tx: mpsc::Sender<CommandMsg>) -> io::Result<()> {
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
                Ok(Some(frame)) => {
                    // one-time response channel for this specific command
                    // Oneshot is like FnOnce not buffer size; use in request and response
                    let (reply_tx, reply_rx) = oneshot::channel();

                    // BACK-PRESSURE HAPPENS HERE:
                    // If the Engine(Server) queue has 1024 items(CommandMSg), this .send().await pauses this entire task loop.
                    // The task stops reading from `stream`, forcing the client's TCP stack to stall.
                    if tx.send(CommandMsg { frame, respond_to: reply_tx }).await.is_err() {
                        return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Engine shutdown"));
                    }
                    // Wait for the single-threaded engine to finish our job
                    if let Ok(response_frame) = reply_rx.await {
                        response_frame.encode(&mut output_buffer);
                    }
                    // execute(frame).encode(&mut output_buffer);
                }
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
