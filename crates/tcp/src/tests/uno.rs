use super::prelude::*;

#[tokio::test]
async fn uno_room_uses_the_shared_tcp_transport_and_private_hands() {
    let room = RoomId(1080);
    let rules = UnoRuleSet::default();
    let session = HostSession::new(
        room,
        GameSetup::Uno {
            host_port: 52303,
            rules,
            shuffled_deck: leocard_uno::build_deck(),
        },
    )
    .unwrap();
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
