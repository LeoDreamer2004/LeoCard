use super::support::{Peer, command, join, server, websocket};
use futures_util::{SinkExt, StreamExt};
use leocard_game_common::PlayerId;
use leocard_protocol::{ChatContent, ClientCommand, MAX_FRAME_PAYLOAD, ServerEvent, encode_frame};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn tcp_ws_and_verified_wss_share_lobby_chat_game_and_exit() {
    let server = server().await;
    let address = server.local_addr();
    let mut host = Peer::tcp(address).await;
    host.send(join(1)).await;
    host.until(|event| matches!(event, ServerEvent::LobbySnapshot(_)))
        .await;
    let mut web = Peer::ws(address, false).await;
    web.send(join(2)).await;
    web.until(|event| matches!(event, ServerEvent::LobbySnapshot(_)))
        .await;
    let mut secure = Peer::ws(address, true).await;
    secure.send(join(3)).await;
    let lobby = secure
        .until(|event| matches!(event, ServerEvent::LobbySnapshot(_)))
        .await;
    let ServerEvent::LobbySnapshot(lobby) = lobby.event else {
        unreachable!()
    };
    assert_eq!(lobby.players.len(), 3);

    web.send(command(2, ClientCommand::SetReady { ready: true }))
        .await;
    secure
        .send(command(2, ClientCommand::SetReady { ready: true }))
        .await;
    host.until(|event| matches!(event, ServerEvent::LobbySnapshot(lobby) if lobby.players.iter().all(|player| player.ready))).await;
    host.send(command(2, ClientCommand::StartGame)).await;
    let native_game = host
        .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
        .await;
    let web_game = web
        .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
        .await;
    secure
        .until(|event| matches!(event, ServerEvent::GameSnapshot(_)))
        .await;
    let (ServerEvent::GameSnapshot(native), ServerEvent::GameSnapshot(web_snapshot)) =
        (native_game.event, web_game.event)
    else {
        unreachable!()
    };
    assert_eq!(native.match_id(), web_snapshot.match_id());

    let text = "TCP / WS / WSS 互通";
    secure
        .send(command(
            3,
            ClientCommand::Chat {
                content: ChatContent::Text(text.into()),
            },
        ))
        .await;
    let chat = |event: &ServerEvent| matches!(event, ServerEvent::ChatMessage(message) if message.content == ChatContent::Text(text.into()));
    host.until(chat).await;
    web.until(chat).await;
    secure.until(chat).await;
    secure.send(command(4, ClientCommand::LeaveRoom)).await;
    secure
        .until(|event| matches!(event, ServerEvent::LeftRoom))
        .await;
    host.send(command(3, ClientCommand::CloseRoom)).await;
    host.until(|event| matches!(event, ServerEvent::RoomClosed))
        .await;
    // All transports must deliver the final room event before closing the socket.
    web.until(|event| matches!(event, ServerEvent::RoomClosed))
        .await;
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn malformed_websocket_and_stalled_handshake_do_not_block_other_players() {
    let server = server().await;
    let address = server.local_addr();
    let _stalled = TcpStream::connect(address).await.unwrap();
    let mut malformed = websocket(address, false).await;
    malformed
        .send(Message::Binary(vec![0, 0, 0, 8, 1].into()))
        .await
        .unwrap();
    assert!(
        timeout(Duration::from_secs(2), malformed.next())
            .await
            .unwrap()
            .is_none_or(|message| message.is_err() || matches!(message, Ok(Message::Close(_))))
    );

    let mut oversized = websocket(address, false).await;
    let _ = oversized
        .send(Message::Binary(vec![0; MAX_FRAME_PAYLOAD + 5].into()))
        .await;
    let result = timeout(Duration::from_secs(2), oversized.next())
        .await
        .unwrap();
    assert!(
        result.is_none_or(|message| message.is_err() || matches!(message, Ok(Message::Close(_))))
    );
    let mut host = Peer::tcp(address).await;
    host.send(join(1)).await;
    host.until(|event| matches!(event, ServerEvent::Joined { .. }))
        .await;
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn websocket_control_frames_and_kick_keep_authoritative_messages_intact() {
    let server = server().await;
    let address = server.local_addr();
    let mut host = Peer::tcp(address).await;
    host.send(join(1)).await;
    host.until(|event| matches!(event, ServerEvent::Joined { .. }))
        .await;
    let mut socket = websocket(address, false).await;
    socket
        .send(Message::Ping(vec![1, 2, 3].into()))
        .await
        .unwrap();
    assert_eq!(
        timeout(Duration::from_secs(2), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap(),
        Message::Pong(vec![1, 2, 3].into())
    );
    socket
        .send(Message::Binary(encode_frame(&join(2)).unwrap().into()))
        .await
        .unwrap();
    let mut guest = Peer::WebSocket(Box::new(socket));
    guest
        .until(|event| matches!(event, ServerEvent::LobbySnapshot(_)))
        .await;
    host.send(command(
        2,
        ClientCommand::KickPlayer {
            player: PlayerId(1),
        },
    ))
    .await;
    guest
        .until(|event| matches!(event, ServerEvent::KickedFromRoom))
        .await;
    server.shutdown().await.unwrap();
}
