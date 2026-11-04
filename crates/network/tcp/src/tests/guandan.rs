use super::prelude::*;
use leocard_game_common::PlayerId;
use leocard_guandan::{GuandanCard, GuandanRuleSet, build_deck};
use std::collections::HashSet;

#[tokio::test]
async fn four_clients_receive_only_their_own_guandan_hands() {
    let room = RoomId(108);
    let session = HostSession::new(
        room,
        GameSetup::Guandan {
            host_port: 10800,
            rules: GuandanRuleSet::default(),
            shuffled_deck: build_deck(),
        },
    )
    .unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let mut clients = Vec::new();
    let mut player_ids = Vec::new();

    for index in 0..4_u64 {
        let mut client = TcpClient::connect(server.local_addr()).await.unwrap();
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
                    avatar_png: None,
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

    let mut dealt = HashSet::<GuandanCard>::new();
    for (expected_player, client) in player_ids.into_iter().zip(&mut clients) {
        let snapshot = timeout(Duration::from_secs(2), async {
            loop {
                if let ServerEvent::GameSnapshot(GameSnapshot::Guandan(snapshot)) =
                    client.receive().await.unwrap().event
                {
                    return snapshot;
                }
            }
        })
        .await
        .expect("server should send a Guandan snapshot");
        assert_eq!(snapshot.you, expected_player);
        assert_eq!(snapshot.your_hand.len(), 27);
        assert_eq!(snapshot.players.len(), 4);
        assert!(snapshot.players.iter().all(|player| player.hand_len == 27));
        for card in snapshot.your_hand {
            assert!(dealt.insert(card), "the same card appeared in two hands");
        }
    }
    assert_eq!(dealt.len(), 108);

    server.shutdown().await.unwrap();
}
