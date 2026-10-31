mod kicking;

use super::connection::complete_handshake;
use super::state::{EventQueue, NetworkEvent, NetworkLaunch, normalize_server_address};
use super::worker::run_network;
use super::{LocalPlayerConnection, NETWORK_ROOM_ID, NetworkState, TcpGameClient};
use crate::ClientModel;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    AVATAR_DIMENSION, ClientCommand, ClientMessage, GameRules, GameSnapshot, LobbySnapshot,
    PROTOCOL_VERSION, RequestId, Revision, SeatId, ServerEvent, ServerMessage,
};

use leocard_qigui523::QiGuiRuleSet;
use leocard_tcp::{TcpClient, read_message, write_message};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};

async fn confirm_join(stream: &mut TcpStream) {
    let _: ClientMessage = read_message(stream).await.unwrap();
    write_message(stream, &message(ServerEvent::Joined { you: PlayerId(0) }))
        .await
        .unwrap();
    write_message(stream, &snapshot()).await.unwrap();
}

#[test]
fn join_address_accepts_dns_names_and_ip_literals() {
    assert_eq!(
        normalize_server_address("frp-off.com:52436").unwrap(),
        "frp-off.com:52436"
    );
    assert_eq!(
        normalize_server_address(" 192.168.1.8:52300 ").unwrap(),
        "192.168.1.8:52300"
    );
    assert_eq!(
        normalize_server_address("[2001:db8::1]:52300").unwrap(),
        "[2001:db8::1]:52300"
    );
}

#[test]
fn join_address_rejects_missing_or_invalid_ports() {
    for address in [
        "frp-off.com",
        "frp-off.com:0",
        "frp-off.com:70000",
        ":52300",
    ] {
        assert!(normalize_server_address(address).is_err(), "{address}");
    }
}

#[tokio::test]
async fn leaving_during_reconnect_cancels_a_stalled_handshake() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let (commands, receiver) = tokio::sync::mpsc::channel(8);
    let worker = tokio::spawn(async move {
        run_network(
            NetworkLaunch::Join { address },
            receiver,
            &Arc::new(Mutex::new(Default::default())),
            ClientMessage::new(NETWORK_ROOM_ID, RequestId(1), ClientCommand::Ping),
        )
        .await
    });
    let (mut initial, _) = listener.accept().await.unwrap();
    confirm_join(&mut initial).await;
    drop(initial);
    let (_stalled, _) = tokio::time::timeout(Duration::from_secs(3), listener.accept())
        .await
        .unwrap()
        .unwrap();
    drop(commands);
    tokio::time::timeout(Duration::from_secs(1), worker)
        .await
        .expect("leaving must cancel the pending reconnect handshake")
        .unwrap()
        .unwrap();
}

fn unused_local_port() -> u16 {
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for(client: &mut TcpGameClient, predicate: impl Fn(&ClientModel) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        client.poll();
        if predicate(client.model()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out; network state is {:?}", client.state());
}

#[test]
fn real_tcp_host_and_two_joiners_reach_the_same_three_player_game() {
    let port = unused_local_port();
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let mut host = TcpGameClient::host_with_profile(
        port,
        rules.into(),
        LocalPlayerConnection::temporary("房主", None).unwrap(),
    )
    .unwrap();
    wait_for(&mut host, |model| model.lobby().is_some());
    host.send(ClientCommand::SelectSeat { seat: SeatId(0) });
    wait_for(&mut host, |model| {
        model
            .lobby()
            .is_some_and(|lobby| lobby.players[0].seat == Some(SeatId(0)))
    });

    let address = format!("127.0.0.1:{port}");
    let mut guest_a = TcpGameClient::join("甲", &address).unwrap();
    wait_for(&mut guest_a, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 2)
    });
    guest_a.send(ClientCommand::SelectSeat { seat: SeatId(5) });
    wait_for(&mut guest_a, |model| {
        model
            .lobby()
            .is_some_and(|lobby| lobby.players[1].seat == Some(SeatId(5)))
    });
    let mut guest_b = TcpGameClient::join("乙", &address).unwrap();
    wait_for(&mut host, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });
    wait_for(&mut guest_a, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });
    wait_for(&mut guest_b, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 3)
    });

    assert_eq!(host.model().you(), Some(PlayerId(0)));
    assert_eq!(guest_a.model().room_id(), NETWORK_ROOM_ID);
    assert_eq!(guest_b.model().room_id(), NETWORK_ROOM_ID);
    assert_eq!(host.model().host_port(), Some(port));
    assert_eq!(guest_a.model().host_port(), Some(port));
    assert_eq!(guest_b.model().host_port(), Some(port));

    // Seats reverse join order to exercise the player ID remapping at game start.
    guest_b.send(ClientCommand::SelectSeat { seat: SeatId(2) });
    host.send(ClientCommand::SetReady { ready: true });
    guest_a.send(ClientCommand::SetReady { ready: true });
    guest_b.send(ClientCommand::SetReady { ready: true });
    wait_for(&mut host, |model| {
        model
            .lobby()
            .is_some_and(|lobby| lobby.players.iter().all(|player| player.ready))
    });
    host.send(ClientCommand::StartGame);
    wait_for(&mut host, |model| {
        model
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .is_some()
    });
    wait_for(&mut guest_a, |model| {
        model
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .is_some()
    });
    wait_for(&mut guest_b, |model| {
        model
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .is_some()
    });
    assert_eq!(guest_a.model().you(), Some(PlayerId(2)));
    assert_eq!(guest_b.model().you(), Some(PlayerId(1)));
    let mut observer = TcpGameClient::join("观战", &address).unwrap();
    wait_for(&mut observer, |model| {
        model
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .is_some()
    });
    assert!(observer.model().is_spectating());
    assert_eq!(
        observer.model().game_rules().and_then(GameRules::qigui523),
        Some(&rules)
    );
    let viewed = observer
        .model()
        .game_snapshot()
        .and_then(GameSnapshot::qigui523)
        .unwrap();
    assert_ne!(observer.model().you(), Some(viewed.you));
    assert_eq!(viewed.your_hand.len(), usize::from(rules.hand_size));
    assert!(!observer.model().active_game_meta().unwrap().is_host());
    let target = guest_a.model().you().unwrap();
    assert!(observer.send(ClientCommand::SpectatePlayer { player: target }));
    wait_for(&mut observer, |model| {
        model
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .is_some_and(|game| game.you == target)
    });
    assert_eq!(
        observer
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .unwrap()
            .your_hand,
        guest_a
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .unwrap()
            .your_hand
    );
    assert!(observer.model().is_spectating());
    assert_eq!(
        observer
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .unwrap()
            .players
            .len(),
        3
    );
    assert!(!observer.send(ClientCommand::SetReady { ready: true }));
    assert_eq!(
        guest_a
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .unwrap()
            .host_port,
        port
    );
    assert_eq!(
        host.model()
            .game_snapshot()
            .and_then(GameSnapshot::qigui523)
            .unwrap()
            .your_hand
            .len(),
        usize::from(rules.hand_size)
    );

    // Replacing an uploaded image must reach the uploader, peers and spectators.
    let player = guest_a.model().you().unwrap();
    for shade in [40, 200] {
        let image = RgbaImage::from_pixel(
            AVATAR_DIMENSION,
            AVATAR_DIMENSION,
            Rgba([shade, 80, 160, 255]),
        );
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();
        assert!(guest_a.send(ClientCommand::SetAvatar { png: png.clone() }));
        for viewer in [&mut host, &mut guest_a, &mut guest_b, &mut observer] {
            wait_for(viewer, |model| {
                model
                    .player_avatar(player)
                    .and_then(|id| model.avatars().get(&id))
                    == Some(&png)
            });
        }
    }
}

#[test]
fn host_close_room_event_stops_guests_without_reconnecting() {
    let port = unused_local_port();
    let mut host = TcpGameClient::host_with_profile(
        port,
        QiGuiRuleSet::default().into(),
        LocalPlayerConnection::temporary("房主", None).unwrap(),
    )
    .unwrap();
    wait_for(&mut host, |model| model.lobby().is_some());

    let mut guest = TcpGameClient::join("访客", &format!("127.0.0.1:{port}")).unwrap();
    wait_for(&mut guest, |model| model.lobby().is_some());
    wait_for(&mut host, |model| {
        model.lobby().is_some_and(|lobby| lobby.players.len() == 2)
    });

    assert!(host.send(ClientCommand::CloseRoom));
    wait_for(&mut host, ClientModel::room_closed);
    wait_for(&mut guest, ClientModel::room_closed);

    assert!(!matches!(host.state(), NetworkState::Reconnecting(_)));
    assert!(!matches!(guest.state(), NetworkState::Reconnecting(_)));
}

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
        entrance: None,
    }))
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
