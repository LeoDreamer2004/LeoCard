use super::ClientModel;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ActionClock, ActionTimingSnapshot, GameKind, GameRules, LobbySnapshot, MatchId,
    PROTOCOL_VERSION, Revision, RoomId, ServerEvent, ServerMessage,
};

fn heartbeat(elapsed_millis: u64) -> ServerMessage {
    ServerMessage {
        protocol_version: PROTOCOL_VERSION,
        room_id: RoomId(1),
        revision: Revision(2),
        game_context: None,
        in_reply_to: None,
        action_timing: Some(ActionTimingSnapshot {
            game: GameKind::Uno,
            match_id: MatchId([1; 16]),
            clocks: vec![ActionClock {
                player: PlayerId(1),
                elapsed_millis,
            }],
        }),
        event: ServerEvent::Heartbeat,
    }
}

#[test]
fn invalid_messages_do_not_replace_timing_or_advance_receipt() {
    let mut model = ClientModel::new(RoomId(1));
    let accepted = heartbeat(1_200);
    assert!(model.apply(accepted.clone()));
    let serial = model.action_timing_serial();
    let mut wrong_version = heartbeat(9_000);
    wrong_version.protocol_version = PROTOCOL_VERSION.wrapping_add(1);
    let mut wrong_room = heartbeat(9_000);
    wrong_room.room_id = RoomId(2);
    let mut old = heartbeat(9_000);
    old.revision = Revision(1);
    for rejected in [wrong_version, wrong_room, old] {
        assert!(!model.apply(rejected));
        assert_eq!(model.action_timing(), accepted.action_timing.as_ref());
        assert_eq!(model.action_timing_serial(), serial);
    }
    // A heartbeat at the current revision is still a new calibration.
    assert!(model.apply(heartbeat(1_200)));
    assert_eq!(model.action_timing_serial(), serial + 1);
    assert!(model.apply(heartbeat(2_200)));
    assert_eq!(
        model.action_timing(),
        heartbeat(2_200).action_timing.as_ref()
    );
}

#[test]
fn absent_timing_lobby_and_room_exit_clear_clocks() {
    let lobby = ServerEvent::LobbySnapshot(LobbySnapshot {
        game: GameKind::Uno,
        rules: GameRules::Uno(Default::default()),
        host_port: 1234,
        host: None,
        players: vec![],
        entrance: None,
    });
    for event in [
        lobby,
        ServerEvent::LeftRoom,
        ServerEvent::RoomClosed,
        ServerEvent::KickedFromRoom,
    ] {
        let mut model = ClientModel::new(RoomId(1));
        assert!(model.apply(heartbeat(1_200)));
        let mut message = heartbeat(2_200);
        message.event = event;
        assert!(model.apply(message));
        assert!(model.action_timing().is_none());
        assert!(model.apply(heartbeat(3_200)));
        assert!(model.action_timing().is_none());
    }
    let mut model = ClientModel::new(RoomId(1));
    assert!(model.apply(heartbeat(1_200)));
    let mut stopped = heartbeat(0);
    stopped.action_timing = None;
    assert!(model.apply(stopped));
    assert!(model.action_timing().is_none());
}
