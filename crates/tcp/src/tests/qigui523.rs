use super::prelude::*;

#[tokio::test]
async fn three_clients_join_ready_and_start_over_real_tcp() {
    let room = RoomId(523);
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let session = HostSession::new(
        room,
        GameSetup::QiGui523 {
            host_port: 52300,
            rules,
            shuffled_deck: build_deck(1),
        },
    )
    .unwrap();
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
