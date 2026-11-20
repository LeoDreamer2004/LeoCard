use leocard_protocol::FrameError;
use std::{fmt, io};
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub enum TcpError {
    Io(io::Error),
    Frame(FrameError),
    WebSocket(Box<tungstenite::Error>),
    Tls(String),
    ConnectionClosed,
    WriterClosed,
    Timeout(&'static str),
    ServerTask(String),
}

impl fmt::Display for TcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "TCP I/O error: {error}"),
            Self::Frame(error) => error.fmt(f),
            Self::WebSocket(error) => write!(f, "WebSocket error: {error}"),
            Self::Tls(error) => write!(f, "TLS configuration error: {error}"),
            Self::ConnectionClosed => f.write_str("TCP connection closed"),
            Self::WriterClosed => f.write_str("TCP writer task closed"),
            Self::Timeout(operation) => write!(f, "TCP {operation} timed out"),
            Self::ServerTask(error) => write!(f, "TCP server task failed: {error}"),
        }
    }
}

impl std::error::Error for TcpError {}

impl From<io::Error> for TcpError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<FrameError> for TcpError {
    fn from(value: FrameError) -> Self {
        Self::Frame(value)
    }
}

impl From<tungstenite::Error> for TcpError {
    fn from(value: tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(value))
    }
}
