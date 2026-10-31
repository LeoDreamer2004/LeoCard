use super::prelude::*;
use leocard_game_common::PlayerId;

#[tokio::test]
async fn kicked_client_receives_the_final_notice_before_the_socket_closes() {
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
        assert_eq!(receive_joined(&mut client).await, PlayerId(index));
        clients.push(client);
    }
    clients[0]
        .send(&ClientMessage::new(
            room,
            RequestId(2),
            ClientCommand::KickPlayer {
                player: PlayerId(1),
            },
        ))
        .await
        .unwrap();
    timeout(Duration::from_secs(2), async {
        loop {
            if matches!(
                clients[1].receive().await.unwrap().event,
                ServerEvent::KickedFromRoom
            ) {
                break;
            }
        }
    })
    .await
    .expect("kick notice must be flushed");
    assert!(
        timeout(Duration::from_secs(2), clients[1].receive())
            .await
            .unwrap()
            .is_err()
    );
    for index in [0, 2] {
        timeout(Duration::from_secs(2), async {
            loop {
                if matches!(clients[index].receive().await.unwrap().event, ServerEvent::PlayerKicked { name } if name == "玩家1") { break; }
            }
        }).await.expect("remaining players see the announcement");
    }
    server.shutdown().await.unwrap();
}
