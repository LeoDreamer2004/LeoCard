use super::prelude::*;

#[tokio::test]
async fn shengji_room_slow_deals_private_hands_over_the_shared_tcp_transport() {
    let room = RoomId(8080);
    let session = HostSession::new(
        room,
        GameSetup::Shengji {
            host_port: 52302,
            rules: ShengjiRuleSet::default(),
            shuffled_deck: leocard_shengji::build_deck(),
        },
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
