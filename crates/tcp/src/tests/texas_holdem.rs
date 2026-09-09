use super::prelude::*;

#[tokio::test]
async fn texas_room_uses_the_same_tcp_lobby_and_private_snapshots() {
    let room = RoomId(9527);
    let rules = TexasHoldemRuleSet::default();
    let session = HostSession::new(
        room,
        GameSetup::TexasHoldem {
            host_port: 52301,
            rules,
            shuffled_deck: leocard_texas_holdem::build_deck(false),
        },
    )
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
