use super::commands::is_decision;
use super::state::ActionTimingState;
use super::windows::PendingAction;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, MatchId, QiGui523Command, UnoCommand};
use std::time::Duration;

fn window(player: u8, key: u8) -> PendingAction {
    PendingAction {
        player: PlayerId(player),
        key: vec![key],
    }
}

#[test]
fn repeated_snapshots_do_not_reset_elapsed_and_same_player_can_start_a_new_decision() {
    let mut timing = ActionTimingState::default();
    let id = MatchId([1; 16]);
    timing.sync(GameKind::Texas, id, vec![window(0, 1)]);
    timing.advance(Duration::from_millis(1250));
    timing.sync(GameKind::Texas, id, vec![window(0, 1)]);
    assert_eq!(timing.snapshot().unwrap().clocks[0].elapsed_millis, 1250);
    assert_eq!(timing.complete(PlayerId(0)), Some(1250));
    assert_eq!(timing.complete(PlayerId(0)), None);
    timing.sync(GameKind::Texas, id, vec![window(0, 1)]);
    timing.advance(Duration::from_millis(500));
    assert_eq!(timing.complete(PlayerId(0)), Some(500));
}

#[test]
fn simultaneous_decisions_are_independent_and_new_keys_restart_without_a_sample() {
    let mut timing = ActionTimingState::default();
    let id = MatchId([1; 16]);
    timing.sync(GameKind::Mahjong, id, vec![window(0, 1), window(1, 1)]);
    timing.advance(Duration::from_secs(2));
    assert_eq!(timing.complete(PlayerId(0)), Some(2000));
    timing.sync(GameKind::Mahjong, id, vec![window(1, 1)]);
    timing.advance(Duration::from_secs(1));
    assert_eq!(timing.complete(PlayerId(1)), Some(3000));
    timing.sync(GameKind::Mahjong, id, vec![window(1, 2)]);
    timing.advance(Duration::from_secs(4));
    timing.sync(GameKind::Mahjong, id, vec![window(1, 3)]);
    assert_eq!(timing.complete(PlayerId(1)), Some(0));
}

#[test]
fn match_switches_and_leaving_clear_old_clocks() {
    let mut timing = ActionTimingState::default();
    timing.sync(GameKind::Uno, MatchId([1; 16]), vec![window(0, 1)]);
    timing.advance(Duration::from_secs(9));
    timing.sync(GameKind::Uno, MatchId([2; 16]), vec![window(0, 1)]);
    assert_eq!(timing.snapshot().unwrap().clocks[0].elapsed_millis, 0);
    timing.clear();
    assert!(timing.snapshot().is_none());
}

#[test]
fn optional_actions_and_settings_are_not_completed_turns() {
    assert!(is_decision(&QiGui523Command::Pass.into()));
    assert!(is_decision(&UnoCommand::DrawCard.into()));
    assert!(!is_decision(&UnoCommand::CallUno.into()));
    assert!(!is_decision(
        &UnoCommand::SetAutoPlay { enabled: true }.into()
    ));
}
