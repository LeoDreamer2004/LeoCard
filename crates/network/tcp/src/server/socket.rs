use super::session::Inbound;
use super::tls::ServerTls;
use crate::codec::FrameReader;
use crate::{TcpError, WRITE_TIMEOUT, write_message};
use futures_util::{SinkExt, StreamExt};
use leocard_host::ConnectionId;
use leocard_protocol::{MAX_FRAME_PAYLOAD, ServerMessage, decode_frame, encode_frame};
use std::io;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};
use tokio_tungstenite::{WebSocketStream, accept_async_with_config};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PACKET_SIZE: usize = MAX_FRAME_PAYLOAD + 4;

trait SocketIo: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> SocketIo for T {}

type WebSocket = WebSocketStream<Box<dyn SocketIo>>;

enum Socket {
    Tcp(TcpStream),
    WebSocket(Box<WebSocket>),
}

impl Socket {
    async fn accept(stream: TcpStream, tls: Option<ServerTls>) -> Result<Self, TcpError> {
        timeout(HANDSHAKE_TIMEOUT, async {
            let mut first = [0];
            if stream.peek(&mut first).await? == 0 {
                return Err(TcpError::ConnectionClosed);
            }
            // A frame is capped at 1 MiB, so its four-byte length always starts with 0.
            if first[0] == 0 {
                return Ok(Self::Tcp(stream));
            }
            let stream: Box<dyn SocketIo> = match first[0] {
                b'G' => Box::new(stream),
                0x16 => {
                    let tls =
                        tls.ok_or_else(|| TcpError::Tls("房主尚未配置 WSS 证书".to_owned()))?;
                    Box::new(tls.acceptor.accept(stream).await?)
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "expected LeoCard TCP, WebSocket or TLS handshake",
                    )
                    .into());
                }
            };
            let config = WebSocketConfig::default()
                .read_buffer_size(4096)
                .write_buffer_size(0)
                .max_write_buffer_size(MAX_PACKET_SIZE * 2)
                .max_message_size(Some(MAX_PACKET_SIZE))
                .max_frame_size(Some(MAX_PACKET_SIZE));
            let websocket = accept_async_with_config(stream, Some(config)).await?;
            Ok(Self::WebSocket(Box::new(websocket)))
        })
        .await
        .map_err(|_| TcpError::Timeout("socket handshake"))?
    }
}

pub(super) async fn serve_socket(
    connection: ConnectionId,
    stream: TcpStream,
    mut messages: mpsc::Receiver<ServerMessage>,
    inbound: mpsc::Sender<Inbound>,
    tls: Option<ServerTls>,
) {
    match Socket::accept(stream, tls).await {
        Ok(Socket::Tcp(stream)) => {
            let _ = serve_tcp(connection, stream, &mut messages, &inbound).await;
        }
        Ok(Socket::WebSocket(mut socket)) => {
            let _ = serve_websocket(connection, &mut socket, &mut messages, &inbound).await;
        }
        Err(_) => {}
    }
    let _ = inbound.send(Inbound::Disconnected(connection)).await;
}

async fn serve_tcp(
    connection: ConnectionId,
    stream: TcpStream,
    messages: &mut mpsc::Receiver<ServerMessage>,
    inbound: &mpsc::Sender<Inbound>,
) -> Result<(), TcpError> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = FrameReader::new(reader);
    loop {
        tokio::select! {
            message = reader.receive() => {
                inbound.send(Inbound::Message(connection, message?)).await
                    .map_err(|_| TcpError::ConnectionClosed)?;
            }
            message = messages.recv() => {
                let Some(message) = message else {
                    timeout(WRITE_TIMEOUT, writer.shutdown()).await
                        .map_err(|_| TcpError::Timeout("socket shutdown"))??;
                    return Ok(());
                };
                write_message(&mut writer, &message).await?;
            }
        }
    }
}

/// Each binary WebSocket message carries one complete, unchanged LeoCard frame.
async fn serve_websocket(
    connection: ConnectionId,
    socket: &mut WebSocket,
    messages: &mut mpsc::Receiver<ServerMessage>,
    inbound: &mpsc::Sender<Inbound>,
) -> Result<(), TcpError> {
    loop {
        tokio::select! {
            message = socket.next() => {
                match message.ok_or(TcpError::ConnectionClosed)?? {
                    Message::Binary(frame) => {
                        let message = decode_frame(&frame)?;
                        inbound.send(Inbound::Message(connection, message)).await
                            .map_err(|_| TcpError::ConnectionClosed)?;
                    }
                    Message::Ping(_) | Message::Close(_) => {
                        // Tungstenite queues the protocol response while reading.
                        socket_write(socket.flush()).await?;
                    }
                    Message::Pong(_) => {}
                    _ => return Err(io::Error::new(
                        io::ErrorKind::InvalidData, "LeoCard expects binary WebSocket messages",
                    ).into()),
                }
            }
            message = messages.recv() => {
                let Some(message) = message else {
                    socket_write(socket.close(None)).await?;
                    return Ok(());
                };
                let frame = encode_frame(&message)?;
                socket_write(socket.send(Message::Binary(frame.into()))).await?;
            }
        }
    }
}

async fn socket_write<T>(
    operation: impl Future<Output = Result<T, WebSocketError>>,
) -> Result<T, TcpError> {
    timeout(WRITE_TIMEOUT, operation)
        .await
        .map_err(|_| TcpError::Timeout("WebSocket write"))?
        .map_err(TcpError::from)
}
