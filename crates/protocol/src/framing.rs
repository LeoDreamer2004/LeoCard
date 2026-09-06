use crate::MAX_FRAME_PAYLOAD;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug)]
pub enum FrameError {
    Serialize(postcard::Error),
    Deserialize(postcard::Error),
    FrameTooShort,
    PayloadTooLarge { actual: usize, maximum: usize },
    LengthMismatch { declared: usize, actual: usize },
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(f, "failed to serialize frame: {error}"),
            Self::Deserialize(error) => write!(f, "failed to deserialize frame: {error}"),
            Self::FrameTooShort => f.write_str("frame is shorter than its 4-byte length prefix"),
            Self::PayloadTooLarge { actual, maximum } => {
                write!(f, "frame payload is {actual} bytes; maximum is {maximum}")
            }
            Self::LengthMismatch { declared, actual } => {
                write!(
                    f,
                    "frame declares {declared} payload bytes but contains {actual}"
                )
            }
        }
    }
}

impl std::error::Error for FrameError {}

/// 编码为可直接写入 TCP 的一帧：4 字节大端负载长度 + Postcard 负载。
pub fn encode_frame<T: Serialize>(value: &T) -> Result<Vec<u8>, FrameError> {
    let payload = postcard::to_allocvec(value).map_err(FrameError::Serialize)?;
    if payload.len() > MAX_FRAME_PAYLOAD {
        return Err(FrameError::PayloadTooLarge {
            actual: payload.len(),
            maximum: MAX_FRAME_PAYLOAD,
        });
    }
    let payload_len = u32::try_from(payload.len()).expect("maximum payload fits in u32");
    let mut frame = Vec::with_capacity(payload.len() + 4);
    frame.extend_from_slice(&payload_len.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// 解码一整帧。TCP 实现应先 `read_exact(4)`，校验长度，再 `read_exact(length)`。
pub fn decode_frame<'a, T: Deserialize<'a>>(frame: &'a [u8]) -> Result<T, FrameError> {
    let prefix: [u8; 4] = frame
        .get(..4)
        .ok_or(FrameError::FrameTooShort)?
        .try_into()
        .expect("slice length was checked");
    let declared = u32::from_be_bytes(prefix) as usize;
    if declared > MAX_FRAME_PAYLOAD {
        return Err(FrameError::PayloadTooLarge {
            actual: declared,
            maximum: MAX_FRAME_PAYLOAD,
        });
    }
    let payload = &frame[4..];
    if payload.len() != declared {
        return Err(FrameError::LengthMismatch {
            declared,
            actual: payload.len(),
        });
    }
    postcard::from_bytes(payload).map_err(FrameError::Deserialize)
}
