use super::packet::{AnnouncementPacket, MAX_PACKET};
use super::query_room_info;
use crate::TcpServerHandle;
use ed25519_dalek::{Signer, SigningKey};
use leocard_host::{ConnectionId, GameSetup, HostSession};
use leocard_protocol::{
    AvatarId, ClientCommand, ClientMessage, GameKind, JoinRequest, PlayerGameProfiles, ProfileId,
    ReconnectToken, RequestId, RoomId, RoomInfo, SeatId, ServerEvent, join_identity_payload,
};
use leocard_qigui523::QiGuiRuleSet;

#[test]
fn announcements_are_bounded_and_do_not_broadcast_avatars() {
    let info = RoomInfo {
        host_name: "房主".to_owned(),
        host_avatar: Some(AvatarId(7)),
        avatar_png: Some(vec![1; 8192]),
        game: GameKind::Uno,
        playing: true,
    };
    let bytes = AnnouncementPacket::encode(19, RoomId(8), 54321, info.clone()).unwrap();
    assert!(bytes.len() < MAX_PACKET);
    let decoded = AnnouncementPacket::decode(&bytes).unwrap();
    assert_eq!(decoded.port, 54321);
    assert_eq!(decoded.server_id, 19);
    assert_eq!(decoded.info.avatar_png, None);
    assert_eq!(decoded.info.host_avatar, info.host_avatar);
    assert!(AnnouncementPacket::decode(b"other application").is_none());
    assert!(AnnouncementPacket::decode(&vec![0; MAX_PACKET + 1]).is_none());
    let invalid = AnnouncementPacket::encode(19, RoomId(8), 0, info).unwrap();
    assert!(AnnouncementPacket::decode(&invalid).is_none());
}

#[tokio::test]
async fn discovery_is_read_only_and_reports_the_actual_room_state() {
    let room = RoomId(73);
    let mut session = HostSession::new(
        room,
        GameSetup::shuffled(52300, QiGuiRuleSet::default().into()),
    )
    .unwrap();
    assert!(session.room_info().is_none());
    for index in 0..3 {
        let connection = ConnectionId(100 + index);
        session.handle(
            connection,
            ClientMessage::new(room, RequestId(1), join(room, index as u8)),
        );
        session.handle(
            connection,
            ClientMessage::new(
                room,
                RequestId(2),
                ClientCommand::SelectSeat {
                    seat: SeatId(index as u8),
                },
            ),
        );
        session.handle(
            connection,
            ClientMessage::new(room, RequestId(3), ClientCommand::SetReady { ready: true }),
        );
    }
    let before = session.revision();
    let replies = session.handle(
        ConnectionId(42),
        ClientMessage::new(room, RequestId(0), ClientCommand::RequestRoomInfo),
    );
    assert_eq!(replies.len(), 1);
    assert!(
        matches!(&replies[0].message.event, ServerEvent::RoomInfo(Some(info)) if !info.playing && info.host_name == "玩家0")
    );
    assert_eq!(session.revision(), before);
    assert!(!session.is_current_connection(ConnectionId(42)));
    assert!(session.disconnect(ConnectionId(42)).is_empty());
    assert_eq!(session.revision(), before);
    session.handle(
        ConnectionId(100),
        ClientMessage::new(room, RequestId(4), ClientCommand::StartGame),
    );
    assert!(session.room_info().unwrap().playing);

    let expected = session.room_info().unwrap();
    let server = TcpServerHandle::bind("127.0.0.1:0", session).await.unwrap();
    let info = query_room_info(server.local_addr(), room).await.unwrap();
    assert_eq!(info, expected);
    // A second metadata connection still sees the same host and game.
    assert_eq!(
        query_room_info(server.local_addr(), room).await.unwrap(),
        expected
    );
    server.shutdown().await.unwrap();
}

fn join(room: RoomId, index: u8) -> ClientCommand {
    let key = SigningKey::from_bytes(&[index + 1; 32]);
    let mut request = JoinRequest {
        name: format!("玩家{index}"),
        reconnect_token: ReconnectToken(index as u64 + 1),
        profile_id: ProfileId(key.verifying_key().to_bytes()),
        reference_points: 0,
        completed_games: 0,
        game_profiles: PlayerGameProfiles::default(),
        identity_signature: Vec::new(),
    };
    request.identity_signature = key
        .sign(&join_identity_payload(
            room,
            request.reconnect_token,
            &request.name,
            0,
            0,
            &request.game_profiles,
        ))
        .to_bytes()
        .to_vec();
    ClientCommand::join(request)
}
