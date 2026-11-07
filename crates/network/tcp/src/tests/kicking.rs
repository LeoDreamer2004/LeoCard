use super::prelude::*;
use leocard_game_common::PlayerId;

async fn join(server: &TcpServerHandle, room: RoomId, index: u8) -> (TcpClient, PlayerId) {
    let mut client = TcpClient::connect(server.local_addr()).await.unwrap();
    let key = SigningKey::from_bytes(&[index + 1; 32]);
    let mut join = JoinRequest {
        name: format!("玩家{index}"),
        reconnect_token: ReconnectToken(u64::from(index) + 1),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
        avatar_png: None,
    };
    join.identity_signature = key.sign(&join.identity_payload(room)).to_bytes().to_vec();
    client
        .send(&ClientMessage::new(
            room,
            RequestId(1),
            ClientCommand::join(join),
        ))
        .await
        .unwrap();
    let player = receive_joined(&mut client).await;
    (client, player)
}

async fn assert_kicked(client: &mut TcpClient) {
    timeout(Duration::from_secs(2), async {
        loop {
            if matches!(
                client.receive().await.unwrap().event,
                ServerEvent::KickedFromRoom
            ) {
                break;
            }
        }
    })
    .await
    .expect("kick notice must be flushed");
    assert!(
        timeout(Duration::from_secs(2), client.receive())
            .await
            .unwrap()
            .is_err()
    );
}

#[tokio::test]
async fn midgame_kicked_players_and_spectators_receive_the_final_notice_before_the_socket_closes() {
    let room = RoomId(314);
    let setup = GameSetup::QiGui523 {
        host_port: 52300,
        rules: QiGuiRuleSet::default(),
        shuffled_deck: build_deck(1),
    };
    let host = HostSession::new(room, setup).unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", host).await.unwrap();
    let mut clients = Vec::new();
    for index in 0..3_u8 {
        let (mut client, player) = join(&server, room, index).await;
        assert_eq!(player, PlayerId(index));
        // Rotate join order so game-start IDs always differ from lobby IDs.
        let seat = SeatId((index + 1) % 3);
        client
            .send(&ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat { seat },
            ))
            .await
            .unwrap();
        timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::LobbySnapshot(snapshot) = client.receive().await.unwrap().event
                    && snapshot
                        .players
                        .iter()
                        .any(|entry| entry.player.id == player && entry.seat == Some(seat))
                {
                    break;
                }
            }
        })
        .await
        .expect("server should confirm the selected seat");
        if index > 0 {
            client
                .send(&ClientMessage::new(
                    room,
                    RequestId(3),
                    ClientCommand::SetReady { ready: true },
                ))
                .await
                .unwrap();
            receive_ready(&mut client, player).await;
        }
        clients.push(client);
    }
    clients[0]
        .send(&ClientMessage::new(
            room,
            RequestId(3),
            ClientCommand::StartGame,
        ))
        .await
        .unwrap();
    let mut game_players = Vec::new();
    for client in &mut clients {
        game_players.push(receive_game(client).await.you);
    }
    assert_eq!(game_players, [PlayerId(1), PlayerId(2), PlayerId(0)]);
    let (mut observer, observer_id) = join(&server, room, 3).await;
    assert_eq!(observer_id, PlayerId(128));
    clients[0]
        .send(&ClientMessage::new(
            room,
            RequestId(4),
            ClientCommand::KickPlayer {
                player: observer_id,
            },
        ))
        .await
        .unwrap();
    assert_kicked(&mut observer).await;
    clients[0]
        .send(&ClientMessage::new(
            room,
            RequestId(5),
            ClientCommand::KickPlayer {
                player: game_players[1],
            },
        ))
        .await
        .unwrap();
    assert_kicked(&mut clients[1]).await;
    for index in [0, 2] {
        timeout(Duration::from_secs(2), async {
            loop {
                if matches!(clients[index].receive().await.unwrap().event, ServerEvent::PlayerKicked { name } if name == "玩家1") { break; }
            }
        }).await.expect("remaining players see the announcement");
    }
    server.shutdown().await.unwrap();
}
