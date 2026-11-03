use std::io;

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::protocol::{Frame, decode};

pub struct CommandMsg {
    pub frame: Frame,
    // Oneshot channel allows the engine to send the reply back directly to specific client task
    pub respond_to: oneshot::Sender<Frame>,
}
async fn read_stream_write_input_buff(
    stream: &mut TcpStream,
    input_buffer: &mut BytesMut,
) -> io::Result<usize> {
    let bytes = stream.read_buf(input_buffer).await?; // exact count of valid bytes recieved
    if bytes == 0 {
        if !input_buffer.is_empty() {
            eprintln!(
                "Warning: Client disconnected!!!, left partial frame: {:?}",
                input_buffer
            );
        } else {
            eprintln!("Client Disconnected!!!");
        }
    }
    return Ok(bytes);
}
async fn process_buffer(
    input_buffer: &mut BytesMut,
    output_buffer: &mut BytesMut,
    tx: &mpsc::Sender<CommandMsg>,
) -> io::Result<bool> {
    loop {
        match decode(input_buffer) {
            // ignore empty array
            Ok(Some(Frame::Array(Some(items)))) if items.is_empty() => {
                continue;
            }
            // Execute the frame and store the encoded response in output buffer
            Ok(Some(frame)) => {
                // one-time response channel for this specific command
                // Oneshot is like FnOnce, with no buffer size and only hold one value at a time; use in request and response
                let (reply_tx, reply_rx) = oneshot::channel();

                // BACK-PRESSURE HAPPENS HERE:
                // If the Engine(Server) queue has 1024 items(CommandMSg), this .send().await pauses this entire task loop.
                // The task stops reading from `stream`, forcing the client's TCP stack to stall.
                if tx
                    .send(CommandMsg {
                        frame,
                        respond_to: reply_tx,
                    })
                    .await
                    .is_err()
                {
                    return Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Engine shutdown!",
                    ));
                }
                // Wait for the single-threaded engine to finish our job
                if let Ok(response_frame) = reply_rx.await {
                    response_frame.encode(output_buffer);
                }
                // execute(frame).encode(&mut output_buffer);
            }
            Ok(None) => {
                return Ok(true);
            } // no more bytes
            Err(e) => {
                Frame::Error(format!("Protocol error: {e}")).encode(output_buffer);
                // stream.write_all(&output_buffer).await?;
                return Ok(false);
            }
        }
    }
}
pub async fn handle_stream(mut stream: TcpStream, tx: mpsc::Sender<CommandMsg>) -> io::Result<()> {
    // Dynamically Growable buffer
    let mut input_buffer = BytesMut::new(); // I can use Vec<u8> with read_to_end and take but is messay and complex for persistent loop(for upcoming bytes in the stream)
    let mut output_buffer = BytesMut::new(); // for sending output for commands

    // loop for persistence connection
    loop {
        // read data from stream and write to input buffer
        let bytes = read_stream_write_input_buff(&mut stream, &mut input_buffer).await?;
        if bytes == 0 {
            return Ok(());
        }
        // process buffer
        let should_continue = process_buffer(&mut input_buffer, &mut output_buffer, &tx).await?;

        // One write for all the inputs
        if !output_buffer.is_empty() {
            stream.write_all(&output_buffer).await?;
            output_buffer.clear();
        }
        if !should_continue {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandMsg, process_buffer};
    use crate::protocol::{Frame, decode};
    use bytes::BytesMut;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn incomplete_frame_waits_for_more_bytes() {
        let original = b"*1\r\n$4\r\nPI";
        let mut input_buffer = BytesMut::from(&original[..]);
        let mut output_buffer = BytesMut::new();
        let (tx, _rx) = mpsc::channel::<CommandMsg>(8);

        let should_continue = process_buffer(&mut input_buffer, &mut output_buffer, &tx)
            .await
            .unwrap();

        assert!(should_continue);
        assert!(output_buffer.is_empty());
        assert_eq!(&input_buffer[..], original);
    }

    #[tokio::test]
    async fn invalid_frame_encodes_protocol_error() {
        let mut input_buffer = BytesMut::from(&b"?\r\n"[..]);
        let mut output_buffer = BytesMut::new();
        let (tx, _rx) = mpsc::channel::<CommandMsg>(8);

        let should_continue = process_buffer(&mut input_buffer, &mut output_buffer, &tx)
            .await
            .unwrap();

        assert!(!should_continue);

        match decode(&mut output_buffer).unwrap() {
            Some(Frame::Error(message)) => {
                assert!(message.starts_with("Protocol error:"));
            }
            other => panic!("Expected protocol error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatches_frame_and_encodes_engine_response() {
        let request = b"*1\r\n$4\r\nPING\r\n";
        let mut input_buffer = BytesMut::from(&request[..]);
        let mut output_buffer = BytesMut::new();
        let (tx, mut rx) = mpsc::channel::<CommandMsg>(8);

        // run only process_buffer, not handle_stream
        let task = tokio::spawn(async move {
            let result = process_buffer(&mut input_buffer, &mut output_buffer, &tx).await;

            (result, output_buffer)
        });

        // verify the sent frame to engine
        let command = rx.recv().await.expect("command was not dispatched");

        let mut expected_buffer = BytesMut::from(&request[..]);
        let expected_frame = decode(&mut expected_buffer)
            .unwrap()
            .expect("request should decode");

        assert_eq!(command.frame, expected_frame);

        let response = Frame::Error("test response".to_string());

        assert!(command.respond_to.send(response).is_ok());

        // verify process_buffer encoded the response
        let (result, mut output_buffer) = task.await.unwrap();

        assert!(result.unwrap());

        match decode(&mut output_buffer).unwrap() {
            Some(Frame::Error(message)) => {
                assert_eq!(message, "test response");
            }
            other => panic!("Expected engine response, got {other:?}"),
        }
    }
}
