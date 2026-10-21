use crate::{FRAME_BODY_TIMEOUT, FRAME_HEADER_TIMEOUT, TcpError, WRITE_TIMEOUT};
use leocard_protocol::{FrameError, MAX_FRAME_PAYLOAD, decode_frame, encode_frame};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::{Instant, timeout, timeout_at};

/// Own partial bytes and deadlines so cancelling a receive never loses framing.
pub(crate) struct FrameReader<R> {
    reader: R,
    frame: Vec<u8>,
    received: usize,
    body: bool,
    deadline: Option<Instant>,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            frame: vec![0; 4],
            received: 0,
            body: false,
            deadline: None,
        }
    }

    pub(crate) async fn receive<T: DeserializeOwned>(&mut self) -> Result<T, TcpError> {
        loop {
            if self.received == self.frame.len() {
                if self.body {
                    let result = decode_frame(&self.frame).map_err(TcpError::from);
                    self.frame.resize(4, 0);
                    self.received = 0;
                    self.body = false;
                    self.deadline = None;
                    return result;
                }
                let length =
                    u32::from_be_bytes(self.frame[..4].try_into().expect("frame prefix")) as usize;
                if length > MAX_FRAME_PAYLOAD {
                    return Err(FrameError::PayloadTooLarge {
                        actual: length,
                        maximum: MAX_FRAME_PAYLOAD,
                    }
                    .into());
                }
                self.frame.resize(length + 4, 0);
                self.body = true;
                self.deadline = Some(Instant::now() + FRAME_BODY_TIMEOUT);
                continue;
            }
            let deadline = *self
                .deadline
                .get_or_insert_with(|| Instant::now() + FRAME_HEADER_TIMEOUT);
            let count = timeout_at(deadline, self.reader.read(&mut self.frame[self.received..]))
                .await
                .map_err(|_| {
                    TcpError::Timeout(if self.body {
                        "frame body read"
                    } else {
                        "frame header read"
                    })
                })??;
            if count == 0 {
                return Err(TcpError::ConnectionClosed);
            }
            self.received += count;
        }
    }
}

pub async fn write_message<W: AsyncWrite + Unpin, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<(), TcpError> {
    let frame = encode_frame(message)?;
    timeout(WRITE_TIMEOUT, writer.write_all(&frame))
        .await
        .map_err(|_| TcpError::Timeout("write"))??;
    Ok(())
}

/// Read a complete frame once; callers multiplexing receives must retain a reader.
pub async fn read_message<R: AsyncRead + Unpin, T: DeserializeOwned>(
    reader: &mut R,
) -> Result<T, TcpError> {
    FrameReader::new(reader).receive().await
}
