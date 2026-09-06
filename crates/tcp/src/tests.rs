use super::*;
use ed25519_dalek::{Signer, SigningKey};

use leocard_protocol::{
    ClientCommand, GameSnapshot, JoinRequest, PlayerGameProfiles, PlayerId, ProfileId,
    QiGui523Snapshot, ReconnectToken, RequestId, RoomId, SeatId, ServerEvent, ShengjiSnapshot,
    TexasHoldemSnapshot, UnoSnapshot, join_identity_payload,
};
use leocard_qigui523::{QiGuiRuleSet, build_deck};
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use tokio::time::{Duration, timeout};

async fn receive_joined(client: &mut TcpClient) -> PlayerId {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::Joined { you } = client.receive().await.unwrap().event {
                return you;
            }
        }
    })
    .await
    .expect("server should confirm the join")
}

async fn receive_ready(client: &mut TcpClient, you: PlayerId) {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::LobbySnapshot(snapshot) = client.receive().await.unwrap().event
                && snapshot
                    .players
                    .iter()
                    .any(|player| player.id == you && player.ready)
            {
                return;
            }
        }
    })
    .await
    .expect("server should publish the ready state");
}

async fn receive_game(client: &mut TcpClient) -> QiGui523Snapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(snapshot) = client.receive().await.unwrap().event {
                return snapshot.into_qigui523().expect("七鬼五二三快照");
            }
        }
    })
    .await
    .expect("server should send a game snapshot")
}

async fn receive_texas_game(client: &mut TcpClient) -> TexasHoldemSnapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)) =
                client.receive().await.unwrap().event
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("server should send a Texas Hold'em snapshot")
}

async fn receive_shengji_dealt_card(client: &mut TcpClient) -> ShengjiSnapshot {
    timeout(Duration::from_secs(3), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) =
                client.receive().await.unwrap().event
                && !snapshot.your_hand.is_empty()
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("slow deal should reach every Shengji player")
}

async fn receive_uno_game(client: &mut TcpClient) -> UnoSnapshot {
    timeout(Duration::from_secs(2), async {
        loop {
            if let ServerEvent::GameSnapshot(GameSnapshot::Uno(snapshot)) =
                client.receive().await.unwrap().event
            {
                return snapshot;
            }
        }
    })
    .await
    .expect("server should send an UNO snapshot")
}

#[tokio::test]
async fn three_clients_join_ready_and_start_over_real_tcp() {
    let room = RoomId(523);
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let session = HostSession::qigui523(room, 52300, rules, build_deck(1)).unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let address = server.local_addr();
    let mut clients = Vec::new();
    let mut player_ids = Vec::new();

    for index in 0..3_u64 {
        let mut client = TcpClient::connect(address).await.unwrap();
        let name = format!("P{index}");
        let token = ReconnectToken(index);
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&index.to_be_bytes());
        secret[8] = 1;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = PlayerGameProfiles::default();
        let signature = key
            .sign(&join_identity_payload(
                room,
                token,
                &name,
                0,
                0,
                &game_profiles,
            ))
            .to_bytes()
            .to_vec();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(1),
                ClientCommand::join(JoinRequest {
                    name,
                    reconnect_token: token,
                    profile_id: ProfileId(key.verifying_key().to_bytes()),
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles,
                    identity_signature: signature,
                }),
            ))
            .await
            .unwrap();
        let player_id = receive_joined(&mut client).await;
        client
            .send(&ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ))
            .await
            .unwrap();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(3),
                ClientCommand::SetReady { ready: true },
            ))
            .await
            .unwrap();
        receive_ready(&mut client, player_id).await;
        clients.push(client);
        player_ids.push(player_id);
    }

    let host_index = player_ids
        .iter()
        .position(|player| *player == PlayerId(0))
        .expect("one connected client should be the host");
    clients[host_index]
        .send(&ClientMessage::new(
            room,
            RequestId(4),
            ClientCommand::StartGame,
        ))
        .await
        .unwrap();

    for (expected_player, client) in player_ids.into_iter().zip(&mut clients) {
        let snapshot = receive_game(client).await;
        assert_eq!(snapshot.you, expected_player);
        assert_eq!(snapshot.your_hand.len(), 5);
        assert!(snapshot.players.iter().all(|player| player.hand_len == 5));
    }

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn texas_room_uses_the_same_tcp_lobby_and_private_snapshots() {
    let room = RoomId(9527);
    let rules = TexasHoldemRuleSet::default();
    let session =
        HostSession::texas_holdem(room, 52301, rules, leocard_texas_holdem::build_deck(false))
            .unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let address = server.local_addr();
    let mut clients = Vec::new();
    let mut player_ids = Vec::new();

    for index in 0..3_u64 {
        let mut client = TcpClient::connect(address).await.unwrap();
        let name = format!("T{index}");
        let token = ReconnectToken(index + 100);
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&(index + 100).to_be_bytes());
        secret[8] = 2;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = PlayerGameProfiles::default();
        let signature = key
            .sign(&join_identity_payload(
                room,
                token,
                &name,
                0,
                0,
                &game_profiles,
            ))
            .to_bytes()
            .to_vec();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(1),
                ClientCommand::join(JoinRequest {
                    name,
                    reconnect_token: token,
                    profile_id: ProfileId(key.verifying_key().to_bytes()),
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles,
                    identity_signature: signature,
                }),
            ))
            .await
            .unwrap();
        let player = receive_joined(&mut client).await;
        client
            .send(&ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ))
            .await
            .unwrap();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(3),
                ClientCommand::SetReady { ready: true },
            ))
            .await
            .unwrap();
        receive_ready(&mut client, player).await;
        clients.push(client);
        player_ids.push(player);
    }

    let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
    clients[host]
        .send(&ClientMessage::new(
            room,
            RequestId(4),
            ClientCommand::StartGame,
        ))
        .await
        .unwrap();

    let mut private_hands = Vec::new();
    for (expected, client) in player_ids.into_iter().zip(&mut clients) {
        let snapshot = receive_texas_game(client).await;
        assert_eq!(snapshot.you, expected);
        assert_eq!(snapshot.your_hole_cards.len(), 2);
        assert!(snapshot.revealed_hands.is_empty());
        assert_eq!(snapshot.players.len(), 3);
        private_hands.push(snapshot.your_hole_cards);
    }
    assert_ne!(private_hands[0], private_hands[1]);
    assert_ne!(private_hands[1], private_hands[2]);

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn shengji_room_slow_deals_private_hands_over_the_shared_tcp_transport() {
    let room = RoomId(8080);
    let session = HostSession::shengji(
        room,
        52302,
        ShengjiRuleSet::default(),
        leocard_shengji::build_deck(),
    )
    .unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let address = server.local_addr();
    let mut clients = Vec::new();
    let mut player_ids = Vec::new();

    for index in 0..4_u64 {
        let mut client = TcpClient::connect(address).await.unwrap();
        let name = format!("S{index}");
        let token = ReconnectToken(index + 200);
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&(index + 200).to_be_bytes());
        secret[8] = 3;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = PlayerGameProfiles::default();
        let signature = key
            .sign(&join_identity_payload(
                room,
                token,
                &name,
                0,
                0,
                &game_profiles,
            ))
            .to_bytes()
            .to_vec();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(1),
                ClientCommand::join(JoinRequest {
                    name,
                    reconnect_token: token,
                    profile_id: ProfileId(key.verifying_key().to_bytes()),
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles,
                    identity_signature: signature,
                }),
            ))
            .await
            .unwrap();
        let player = receive_joined(&mut client).await;
        client
            .send(&ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ))
            .await
            .unwrap();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(3),
                ClientCommand::SetReady { ready: true },
            ))
            .await
            .unwrap();
        receive_ready(&mut client, player).await;
        clients.push(client);
        player_ids.push(player);
    }

    let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
    clients[host]
        .send(&ClientMessage::new(
            room,
            RequestId(4),
            ClientCommand::StartGame,
        ))
        .await
        .unwrap();

    for (expected, client) in player_ids.into_iter().zip(&mut clients) {
        let snapshot = receive_shengji_dealt_card(client).await;
        assert_eq!(snapshot.you, expected);
        assert_eq!(snapshot.your_hand.len(), 1);
        assert_eq!(snapshot.players.len(), 4);
        assert_eq!(
            snapshot
                .players
                .iter()
                .find(|player| player.id == expected)
                .map(|player| player.hand_len),
            Some(1)
        );
        let dealt = snapshot
            .players
            .iter()
            .map(|player| player.hand_len)
            .sum::<u8>();
        // 四个客户端是逐个读取的，定时发牌可在相邻读取之间继续推进；这里
        // 验证收到的是该玩家第一轮的私有手牌，而不把全桌时钟钉死在某一拍。
        assert!((expected.0 + 1..=expected.0 + 4).contains(&dealt));
    }

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn uno_room_uses_the_shared_tcp_transport_and_private_hands() {
    let room = RoomId(1080);
    let rules = UnoRuleSet::default();
    let session = HostSession::uno(room, 52303, rules, leocard_uno::build_deck()).unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let address = server.local_addr();
    let mut clients = Vec::new();
    let mut player_ids = Vec::new();

    for index in 0..2_u64 {
        let mut client = TcpClient::connect(address).await.unwrap();
        let name = format!("U{index}");
        let token = ReconnectToken(index + 300);
        let mut secret = [0; 32];
        secret[..8].copy_from_slice(&(index + 300).to_be_bytes());
        secret[8] = 4;
        let key = SigningKey::from_bytes(&secret);
        let game_profiles = PlayerGameProfiles::default();
        let signature = key
            .sign(&join_identity_payload(
                room,
                token,
                &name,
                0,
                0,
                &game_profiles,
            ))
            .to_bytes()
            .to_vec();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(1),
                ClientCommand::join(JoinRequest {
                    name,
                    reconnect_token: token,
                    profile_id: ProfileId(key.verifying_key().to_bytes()),
                    reference_points: 0,
                    completed_games: 0,
                    game_profiles,
                    identity_signature: signature,
                }),
            ))
            .await
            .unwrap();
        let player = receive_joined(&mut client).await;
        client
            .send(&ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ))
            .await
            .unwrap();
        client
            .send(&ClientMessage::new(
                room,
                RequestId(3),
                ClientCommand::SetReady { ready: true },
            ))
            .await
            .unwrap();
        receive_ready(&mut client, player).await;
        clients.push(client);
        player_ids.push(player);
    }

    let host = player_ids.iter().position(|id| *id == PlayerId(0)).unwrap();
    clients[host]
        .send(&ClientMessage::new(
            room,
            RequestId(4),
            ClientCommand::StartGame,
        ))
        .await
        .unwrap();

    let mut hands = Vec::new();
    for (expected, client) in player_ids.into_iter().zip(&mut clients) {
        let snapshot = receive_uno_game(client).await;
        assert_eq!(snapshot.you, expected);
        assert_eq!(snapshot.your_hand.len(), 7);
        assert_eq!(snapshot.players.len(), 2);
        assert!(snapshot.players.iter().all(|player| player.hand_len == 7));
        hands.push(snapshot.your_hand);
    }
    assert!(hands.windows(2).any(|pair| pair[0] != pair[1]));

    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn framed_codec_works_when_bytes_arrive_in_fragments() {
    let (mut left, mut right) = tokio::io::duplex(64);
    let message = ClientMessage::new(RoomId(1), RequestId(1), ClientCommand::RequestSnapshot);
    let frame = encode_frame(&message).unwrap();
    let writer = tokio::spawn(async move {
        for chunk in frame.chunks(2) {
            left.write_all(chunk).await.unwrap();
        }
    });
    let decoded: ClientMessage = read_message(&mut right).await.unwrap();
    writer.await.unwrap();
    assert_eq!(decoded, message);
}
