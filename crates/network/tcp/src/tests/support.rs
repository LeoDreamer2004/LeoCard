use crate::{ServerTls, TcpClient, TcpServerHandle};
use ed25519_dalek::{Signer, SigningKey};
use futures_util::{SinkExt, StreamExt};
use leocard_host::{GameSetup, HostSession};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameRules, JoinRequest, ProfileId, ReconnectToken, RequestId,
    RoomId, ServerEvent, ServerMessage, decode_frame, encode_frame,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName};
use tokio_rustls::rustls::{ClientConfig, RootCertStore, crypto::ring};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, client_async};

pub(super) const ROOM: RoomId = RoomId(523);
pub(super) const CERT: &[u8] = include_bytes!("fixtures/localhost.crt");
// Public test fixture only. Never use this key or certificate in a deployed room.
const KEY: &[u8] = include_bytes!("fixtures/localhost.key");

pub(super) async fn server() -> TcpServerHandle {
    let session = HostSession::new(
        ROOM,
        GameSetup::shuffled(52300, GameRules::Uno(Default::default())),
    )
    .unwrap();
    TcpServerHandle::bind_with_tls(
        "127.0.0.1:0",
        session,
        ServerTls::from_pem(CERT, KEY).unwrap(),
    )
    .await
    .unwrap()
}

pub(super) fn command(request: u64, command: ClientCommand) -> ClientMessage {
    ClientMessage::new(ROOM, RequestId(request), command)
}

pub(super) fn join(seed: u8) -> ClientMessage {
    let key = SigningKey::from_bytes(&[seed; 32]);
    let mut request = JoinRequest {
        name: format!("Socket{seed}"),
        reconnect_token: ReconnectToken(u64::from(seed)),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: Default::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    request.identity_signature = key
        .sign(&request.identity_payload(ROOM))
        .to_bytes()
        .to_vec();
    command(1, ClientCommand::join(request))
}

pub(super) trait TestIo: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> TestIo for T {}

pub(super) type WebSocket = WebSocketStream<Box<dyn TestIo>>;

pub(super) async fn websocket(address: SocketAddr, secure: bool) -> WebSocket {
    let stream = TcpStream::connect(address).await.unwrap();
    let stream: Box<dyn TestIo> = if secure {
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from_pem_slice(CERT).unwrap())
            .unwrap();
        let config = ClientConfig::builder_with_provider(Arc::new(ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        Box::new(
            connector
                .connect(ServerName::try_from("localhost").unwrap(), stream)
                .await
                .unwrap(),
        )
    } else {
        Box::new(stream)
    };
    let scheme = if secure { "wss" } else { "ws" };
    client_async(format!("{scheme}://localhost:{}/", address.port()), stream)
        .await
        .unwrap()
        .0
}

pub(super) enum Peer {
    Tcp(TcpClient),
    WebSocket(Box<WebSocket>),
}

impl Peer {
    pub(super) async fn tcp(address: SocketAddr) -> Self {
        Self::Tcp(TcpClient::connect(address).await.unwrap())
    }

    pub(super) async fn ws(address: SocketAddr, secure: bool) -> Self {
        Self::WebSocket(Box::new(websocket(address, secure).await))
    }

    pub(super) async fn send(&mut self, message: ClientMessage) {
        match self {
            Self::Tcp(peer) => peer.send(&message).await.unwrap(),
            Self::WebSocket(peer) => peer
                .send(Message::Binary(encode_frame(&message).unwrap().into()))
                .await
                .unwrap(),
        }
    }

    pub(super) async fn until(
        &mut self,
        predicate: impl Fn(&ServerEvent) -> bool,
    ) -> ServerMessage {
        timeout(Duration::from_secs(5), async {
            loop {
                let message: ServerMessage = match self {
                    Self::Tcp(peer) => peer.receive().await.unwrap(),
                    Self::WebSocket(peer) => {
                        let Message::Binary(frame) = peer.next().await.unwrap().unwrap() else {
                            continue;
                        };
                        decode_frame(&frame).unwrap()
                    }
                };
                if predicate(&message.event) {
                    return message;
                }
                if let ServerEvent::Rejected { reason } = &message.event {
                    panic!("unexpected rejection: {reason}");
                }
            }
        })
        .await
        .expect("authoritative socket response")
    }
}
