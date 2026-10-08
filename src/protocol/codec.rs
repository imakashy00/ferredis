use bytes::{ Buf, Bytes, BytesMut };

use crate::protocol::{ Frame, frame::{ ProtocolError, error } };

// Cap bulk strings at 512 MB;
const MAX_BULK_LEN: usize = 512 * 1024 * 1024;
const MAX_ARRAY_LEN: usize = 1024 * 1024;

// Decode the frames from the front of the buffer
// Three possibilities
// Ok(Some(frame)) - full frame detected and its bytes are removed from the buffer
// Ok(None)        - not enough data yet; 'buffer is left untouched'
// Err(_)          - malformed input; send error to caller
pub fn decode(buffer: &mut BytesMut) -> Result<Option<Frame>, ProtocolError> {
    match parse(buffer, 0)? {
        Some((frame, consumed)) => {
            buffer.advance(consumed);
            Ok(Some(frame))
        }
        None => Ok(None),
    }
}

pub fn parse(buffer: &[u8], pos: usize) -> Result<Option<(Frame, usize)>, ProtocolError> {
    if pos >= buffer.len() {
        return Ok(None);
    }
    let type_byte = buffer[pos];
    let line_start = pos + 1;
    let Some(line_end) = find_crlf(buffer, line_start) else {
        return Ok(None); // header line not complete yet
    };
    let line = &buffer[line_start..line_end];
    let next = line_end + 2;
    match type_byte {
        b'+' => {
            let s = std::str
                ::from_utf8(line)
                .or_else(|_| { error("Invalid utf-8 in simple String") })?;
            Ok(Some((Frame::Simple(s.to_owned()), next)))
        }
        b'-' => {
            let s = std::str::from_utf8(line).or_else(|_| error("invalid utf-8 in error string"))?;
            Ok(Some((Frame::Error(s.to_owned()), next)))
        }
        b':' => Ok(Some((Frame::Integer(parse_int(line)?), next))),
        b'$' => {
            let length = parse_int(line)?;
            if length == -1 {
                return Ok(Some((Frame::Bulk(None), next)));
            }
            if length < 0 || (length as usize) > MAX_BULK_LEN {
                return error("Invalid bulk length");
            }
            let length = length as usize;
            let end = next + length;
            if buffer.len() < end + 2 {
                return Ok(None); // incomplete data
            }
            if &buffer[end..end + 2] != b"\r\n" {
                return error("Bulk String didn't end with crlf.");
            }
            let data = Bytes::copy_from_slice(&buffer[next..end]);
            Ok(Some((Frame::Bulk(Some(data)), end + 2)))
        }
        b'*' => {
            let length = parse_int(line)?;
            if length == -1 {
                return Ok(Some((Frame::Array(None), next)));
            }
            if length < 0 || (length as usize) > MAX_ARRAY_LEN {
                return error("Invalid multi bulk lenth");
            }
            let length = length as usize;
            let mut items = Vec::with_capacity(length.min(1024));
            let mut curr = next;
            for _ in 0..length {
                // Recursion
                match parse(buffer, curr)? {
                    Some((frame, after)) => {
                        items.push(frame);
                        curr = after;
                    }
                    None => {
                        return Ok(None); // an element is incomplete
                    }
                }
            }
            Ok(Some((Frame::Array(Some(items)), curr)))
        }
        other => error(format!("Unknown byte type:{}", other as char)),
    }
}

fn find_crlf(buffer: &[u8], start: usize) -> Option<usize> {
    buffer[start..]
        .windows(2)
        .position(|w| w == b"\r\n")
        .map(|p| start + p)
}

fn parse_int(line: &[u8]) -> Result<i64, ProtocolError> {
    std::str
        ::from_utf8(line)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .map_or_else(|| error("Invalid integer"), Ok)
}
// Commands arrive as an array of bulk strings: ["SET", "key", "value"]
fn into_args(frame: Frame) -> Option<Vec<Bytes>> {
    match frame {
        Frame::Array(Some(items)) if !items.is_empty() =>
            items
                .into_iter()
                .map(|f| {
                    match f {
                        Frame::Bulk(Some(b)) => Some(b),
                        _ => None,
                    }
                })
                .collect(),
        _ => None,
    }
}

pub fn execute(frame: Frame) -> Frame {
    // Everything should be the array of bulk string
    let Some(args) = into_args(frame) else {
        return Frame::Error("ERR Protocol error: expected array of bulk strings".into());
    };
    match args[0].to_ascii_uppercase().as_slice() {
        b"PING" =>
            match args.get(1) {
                Some(msg) => Frame::Bulk(Some(msg.clone())),
                None => Frame::Simple("PONG".into()),
            }
        b"ECHO" if args.len() == 2 => Frame::Bulk(Some(args[1].clone())),
        // redis-cli sends `COMMAND DOCS` on startup; an empty array keeps it happy 😊
        b"COMMAND" => Frame::Array(Some(vec![])),
        _ => Frame::Error(format!("Unknown command '{}'", String::from_utf8_lossy(&args[0]))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bulk(s: &str) -> Frame {
        Frame::Bulk(Some(Bytes::copy_from_slice(s.as_bytes())))
    }
    #[test]
    fn into_args_accepts_ping() {
        let f = Frame::Array(Some(vec![Frame::Bulk(Some(Bytes::from_static(b"ping")))]));
        assert!(into_args(f).is_some());
    }
    #[test]
    fn single_complete_frame() {
        let mut buf = BytesMut::from(&b"*1\r\n$4\r\nPING\r\n"[..]);
        let frame = decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame, Frame::Array(Some(vec![bulk("PING")])));
        assert!(buf.is_empty());
    }

    #[test]
    fn partial_frame_is_left_untouched_then_completes() {
        let mut buf = BytesMut::from(&b"*2\r\n$4\r\nECHO\r\n$3\r\nhe"[..]);
        let before = buf.len();
        assert!(decode(&mut buf).unwrap().is_none());
        assert_eq!(buf.len(), before); // nothing consumed

        buf.extend_from_slice(b"y\r\n");
        let frame = decode(&mut buf).unwrap().unwrap();
        assert_eq!(frame, Frame::Array(Some(vec![bulk("ECHO"), bulk("hey")])));
        assert!(buf.is_empty());
    }

    #[test]
    fn pipelined_frames_plus_trailing_partial() {
        let mut buf = BytesMut::from(
            &b"*1\r\n$4\r\nPING\r\n*2\r\n$4\r\nECHO\r\n$2\r\nhi\r\n*1\r\n$4\r\nPI"[..]
        );
        assert!(decode(&mut buf).unwrap().is_some()); // PING
        assert!(decode(&mut buf).unwrap().is_some()); // ECHO hi
        assert!(decode(&mut buf).unwrap().is_none()); // partial PING
        assert_eq!(&buf[..], b"*1\r\n$4\r\nPI"); // partial kept for next read
    }

    #[test]
    fn split_inside_crlf() {
        let mut buf = BytesMut::from(&b"*1\r\n$4\r\nPING\r"[..]);
        assert!(decode(&mut buf).unwrap().is_none());
        buf.extend_from_slice(b"\n");
        assert!(decode(&mut buf).unwrap().is_some());
    }

    #[test]
    fn null_bulk_and_empty_bulk() {
        let mut buf = BytesMut::from(&b"*2\r\n$-1\r\n$0\r\n\r\n"[..]);
        assert_eq!(
            decode(&mut buf).unwrap().unwrap(),
            Frame::Array(Some(vec![Frame::Bulk(None), bulk("")]))
        );
    }

    #[test]
    fn bad_input_is_an_error() {
        // a bad length inside a RESP array is a protocol error
        let mut buf = BytesMut::from(&b"*1\r\n$abc\r\n"[..]);
        assert!(decode(&mut buf).is_err());
        let mut buf = BytesMut::from(&b"*1\r\n?oops\r\n"[..]);
        assert!(decode(&mut buf).is_err());
    }

    #[test]
    fn encode_roundtrip() {
        let frame = Frame::Array(
            Some(
                vec![
                    Frame::Simple("OK".into()),
                    Frame::Integer(-5),
                    bulk("hello"),
                    Frame::Bulk(None)
                ]
            )
        );
        let mut out = BytesMut::new();
        frame.encode(&mut out);
        assert_eq!(decode(&mut out).unwrap().unwrap(), frame);
    }
}
