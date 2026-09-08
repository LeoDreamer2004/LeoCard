use super::qigui::score_history_snapshot;
use super::*;
use leocard_protocol::{
    AvatarId, GameRules, GameSnapshot, LobbySnapshot, MatchId, PROTOCOL_VERSION, PlayerId,
    PlayerReferenceChange, RejectReason, Revision, RoomId, ServerEvent, ServerMessage,
};
use leocard_protocol::{GameKind, GamePhaseView, GameViolation, ProfileId, RuleViolation};
use leocard_qigui523::QiGuiRuleSet;

#[test]
fn finished_match_receipt_survives_a_following_lobby_snapshot() {
    let mut model = ClientModel::new(RoomId(7));
    let match_id = MatchId([9; 16]);
    let profile_id = ProfileId([4; 32]);
    let change = PlayerReferenceChange {
        player: PlayerId(0),
        profile_id,
        delta: 2,
    };
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::GameSnapshot(GameSnapshot::QiGui523(score_history_snapshot(
            None,
            GamePhaseView::Finished {
                match_id,
                finisher: PlayerId(0),
                scores: Vec::new(),
                remaining_hands: Vec::new(),
                reference_changes: vec![change],
                captured_hand_points: 0,
            },
        ))),
    }));
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(2),
        in_reply_to: None,
        event: ServerEvent::LobbySnapshot(LobbySnapshot {
            game: GameKind::QiGui523,
            rules: GameRules::QiGui523(QiGuiRuleSet::default()),
            host_port: 52300,
            host: None,
            players: Vec::new(),
        }),
    }));

    assert_eq!(model.last_finished_match(), Some((match_id, &[change][..])));
}

#[test]
fn avatar_payload_is_cached_independently_from_snapshots() {
    let mut model = ClientModel::new(RoomId(7));
    let png = vec![1, 2, 3, 4];
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::AvatarData {
            id: AvatarId(9),
            png: png.clone(),
        },
    }));
    assert_eq!(model.avatars().get(&AvatarId(9)), Some(&png));
    assert!(model.lobby().is_none());
    assert!(model.qigui523_game().is_none());
}

#[test]
fn repeated_identical_rejections_each_advance_the_local_serial() {
    let mut model = ClientModel::new(RoomId(7));
    let rejection = ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::Rejected {
            reason: RejectReason::Game(GameViolation::QiGui523(RuleViolation::InvalidPattern)),
        },
    };

    assert_eq!(model.rejection_serial(), 0);
    assert!(model.apply(rejection.clone()));
    assert_eq!(model.rejection_serial(), 1);
    assert!(model.apply(rejection));
    assert_eq!(model.rejection_serial(), 2);
}

#[test]
fn room_closed_event_is_remembered_by_the_client_model() {
    let mut model = ClientModel::new(RoomId(7));
    assert!(!model.room_closed());
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::RoomClosed,
    }));
    assert!(model.room_closed());
}

#[test]
fn named_leave_notice_and_local_leave_are_remembered_separately() {
    let mut model = ClientModel::new(RoomId(7));
    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::PlayerLeft {
            name: "小明".to_owned(),
        },
    }));
    assert_eq!(model.notice_serial(), 1);
    assert_eq!(model.last_notice(), Some("小明退出了游戏"));
    assert!(!model.left_room());

    assert!(model.apply(ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(2),
        in_reply_to: None,
        event: ServerEvent::LeftRoom,
    }));
    assert!(model.left_room());
    assert!(!model.room_closed());
}
