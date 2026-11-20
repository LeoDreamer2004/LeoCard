use crate::codec::FrameReader;
use crate::{TcpError, write_message};
use leocard_protocol::{ClientMessage, ServerMessage};
use std::time::Duration;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub struct TcpClient {
    reader: FrameReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

/// TCP 客户端的只读半边。拆分后可与命令发送任务并行等待服务端广播。
pub struct TcpClientReader(FrameReader<OwnedReadHalf>);

/// TCP 客户端的只写半边。
pub struct TcpClientWriter(OwnedWriteHalf);

impl TcpClient {
    pub async fn connect(address: impl ToSocketAddrs) -> Result<Self, TcpError> {
        let stream = tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(address))
            .await
            .map_err(|_| TcpError::Timeout("connect"))??;
        stream.set_nodelay(true)?;
        let (reader, writer) = stream.into_split();
        Ok(Self {
            reader: FrameReader::new(reader),
            writer,
        })
    }

    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), TcpError> {
        write_message(&mut self.writer, message).await
    }

    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        self.reader.receive().await
    }

    pub fn into_split(self) -> (TcpClientReader, TcpClientWriter) {
        (TcpClientReader(self.reader), TcpClientWriter(self.writer))
    }
}

impl TcpClientReader {
    pub async fn receive(&mut self) -> Result<ServerMessage, TcpError> {
        self.0.receive().await
    }
}

impl TcpClientWriter {
    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), TcpError> {
        write_message(&mut self.0, message).await
    }
}
