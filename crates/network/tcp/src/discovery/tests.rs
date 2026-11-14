use super::packet::{AnnouncementPacket, MAX_PACKET};
use leocard_protocol::{AvatarId, GameKind, RoomId, RoomInfo};

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
