use super::state::LobbyEntranceState;
use crate::app::runtime::GameProjection;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, GameRules, LobbyEntranceView, LobbySnapshot, ServerEvent};

fn lobby(entrance: Option<LobbyEntranceView>) -> ServerEvent {
    ServerEvent::LobbySnapshot(LobbySnapshot {
        game: GameKind::Uno,
        rules: GameRules::Uno(Default::default()),
        host_port: 52300,
        host: Some(PlayerId(0)),
        players: Vec::new(),
        entrance,
    })
}

#[test]
fn entrance_does_not_replay_on_snapshots_reconnect_or_returning() {
    let entrance = LobbyEntranceView {
        sequence: 1,
        player: PlayerId(0),
        elapsed_millis: 0,
    };
    let mut state = LobbyEntranceState::default();
    state.apply(&lobby(Some(entrance)));
    assert!(state.active());
    state.advance(5.0);
    assert_eq!(state.active.unwrap().elapsed, 0.0);
    state.advance(1.0);
    state.apply(&lobby(Some(entrance)));
    assert_eq!(state.active.unwrap().elapsed, 1.0);
    state.advance(1.4);
    assert!(!state.active());
    state.apply(&ServerEvent::Joined { you: PlayerId(0) });
    state.apply(&lobby(Some(entrance)));
    assert!(!state.active());
    state.apply(&lobby(None));
    state.apply(&lobby(Some(entrance)));
    assert!(!state.active());
    state.apply(&lobby(Some(LobbyEntranceView {
        sequence: 2,
        ..entrance
    })));
    assert!(state.active());
}
