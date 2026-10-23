use super::*;
use leocard_protocol::{
    AvatarId, GameViolation, PROTOCOL_VERSION, PlayerId, PlayerInteraction, PlayerInteractionKind,
    RejectReason, Revision, RoomId, RuleViolation, ServerEvent, ServerMessage,
};

#[test]
fn accepted_events_reach_observers_without_consuming_presentation_queues() {
    let mut model = ClientModel::new(RoomId(7));
    model.apply(ServerMessage {
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(1),
        in_reply_to: None,
        event: ServerEvent::Joined { you: PlayerId(0) },
    });
    let interaction = PlayerInteraction {
        source: PlayerId(1),
        target: PlayerId(0),
        kind: PlayerInteractionKind::Flower,
        seed: 1,
        elapsed_millis: 100,
    };
    let message = ServerMessage {
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(2),
        in_reply_to: None,
        event: ServerEvent::PlayerInteraction(interaction),
    };
    let mut observed = Vec::new();
    assert!(model.apply_with_events(message.clone(), |player, event| {
        observed.push((player, event.event.clone()))
    }));
    assert_eq!(observed, [(Some(PlayerId(0)), message.event.clone())]);
    assert_eq!(model.take_player_interactions(), [interaction]);
    for rejected in [
        ServerMessage {
            room_id: RoomId(99),
            ..message.clone()
        },
        ServerMessage {
            game_context: None,
            protocol_version: 0,
            ..message.clone()
        },
        ServerMessage {
            revision: Revision(1),
            ..message
        },
    ] {
        assert!(
            !model.apply_with_events(rejected, |_, _| panic!("rejected event reached observer"))
        );
    }
}

#[test]
fn avatar_payload_is_cached_independently_from_snapshots() {
    let mut model = ClientModel::new(RoomId(7));
    let png = vec![1, 2, 3, 4];
    assert!(model.apply(ServerMessage {
        game_context: None,
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
        game_context: None,
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
        game_context: None,
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
        game_context: None,
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
        game_context: None,
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(7),
        revision: Revision(2),
        in_reply_to: None,
        event: ServerEvent::LeftRoom,
    }));
    assert!(model.left_room());
    assert!(!model.room_closed());
}
