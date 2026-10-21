use super::connection::complete_handshake;
use super::state::{EventQueue, NETWORK_ROOM_ID, NetworkEvent};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameRules, LobbySnapshot, PROTOCOL_VERSION, PlayerId, RequestId,
    Revision, ServerEvent, ServerMessage,
};
use leocard_tcp::{TcpClient, read_message, write_message};
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};

fn message(event: ServerEvent) -> ServerMessage {
    ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: NETWORK_ROOM_ID,
        revision: Revision(1),
        in_reply_to: None,
        game_context: None,
        event,
    }
}

fn snapshot() -> ServerMessage {
    message(ServerEvent::LobbySnapshot(LobbySnapshot {
        game: leocard_protocol::GameKind::QiGui523,
        rules: GameRules::QiGui523(Default::default()),
        host_port: 52300,
        host: Some(PlayerId(0)),
        players: Vec::new(),
    }))
}

pub(super) async fn confirm_join(stream: &mut TcpStream) {
    let _: ClientMessage = read_message(stream).await.unwrap();
    write_message(stream, &message(ServerEvent::Joined { you: PlayerId(0) }))
        .await
        .unwrap();
    write_message(stream, &snapshot()).await.unwrap();
}

#[tokio::test]
async fn handshake_requires_a_snapshot_after_identity_confirmation() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = TcpClient::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let events: EventQueue = Arc::new(Mutex::new(Default::default()));
    let worker_events = events.clone();
    let handshake = tokio::spawn(async move {
        complete_handshake(
            client,
            &ClientMessage::new(
                NETWORK_ROOM_ID,
                RequestId(1),
                ClientCommand::RequestSnapshot,
            ),
            &worker_events,
        )
        .await
    });
    let (mut server, _) = listener.accept().await.unwrap();
    let _: ClientMessage = read_message(&mut server).await.unwrap();
    write_message(
        &mut server,
        &message(ServerEvent::Joined { you: PlayerId(0) }),
    )
    .await
    .unwrap();
    write_message(&mut server, &message(ServerEvent::Heartbeat))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert!(!handshake.is_finished());
    write_message(&mut server, &snapshot()).await.unwrap();
    assert!(handshake.await.unwrap().is_ok());
    let events = events.lock().unwrap();
    assert!(
        matches!(events.back(), Some(NetworkEvent::Message(message)) if matches!(message.event, ServerEvent::LobbySnapshot(_)))
    );
}

#[tokio::test]
async fn handshake_rejects_a_different_protocol_version() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let client = TcpClient::connect(listener.local_addr().unwrap())
        .await
        .unwrap();
    let handshake = tokio::spawn(async move {
        complete_handshake(
            client,
            &ClientMessage::new(
                NETWORK_ROOM_ID,
                RequestId(1),
                ClientCommand::RequestSnapshot,
            ),
            &Arc::new(Mutex::new(Default::default())),
        )
        .await
    });
    let (mut server, _) = listener.accept().await.unwrap();
    let mut joined = message(ServerEvent::Joined { you: PlayerId(0) });
    joined.protocol_version += 1;
    write_message(&mut server, &joined).await.unwrap();
    assert!(handshake.await.unwrap().is_err());
}
