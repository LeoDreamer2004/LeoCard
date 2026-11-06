use super::super::{TexasChipTableState, TexasChipUpdates};
use super::reconciliation::{assert_balances, choosing_snapshot, complete_snapshot};
use crate::app::runtime::GameProjection;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameEvent, GameSnapshot, ServerEvent, TexasEvent, TexasPhaseView};
use leocard_texas_holdem::{TexasAction, TexasActionStatistics, TexasStreet};

fn event(event: TexasEvent) -> ServerEvent {
    ServerEvent::GameEvent(GameEvent::Texas(event))
}

fn flush(updates: &mut TexasChipUpdates, state: &mut TexasChipTableState) {
    for (snapshot, events) in updates.take() {
        state.observe(&snapshot, events, true);
        assert_balances(state, &snapshot);
    }
}

#[test]
fn action_refund_runout_and_finish_keep_network_order_at_every_frame_boundary() {
    let choosing = choosing_snapshot();
    let complete = complete_snapshot(true);
    let mut betting = choosing.clone();
    betting.phase = TexasPhaseView::Betting {
        street: TexasStreet::PreFlop,
    };
    let mut raised = betting.clone();
    raised.players[2].stack -= 2;
    raised.players[2].committed_street += 2;
    raised.players[2].committed_total += 2;
    raised.pot += 2;
    let messages = vec![
        event(TexasEvent::ActionApplied {
            player: PlayerId(2),
            action: TexasAction::RaiseTo(12),
            amount: 2,
            statistics: TexasActionStatistics {
                street: TexasStreet::PreFlop,
                all_in_amount: 0,
                full_raise: true,
                raised: true,
                bet_level: 1,
            },
        }),
        ServerEvent::GameSnapshot(GameSnapshot::Texas(raised)),
        event(TexasEvent::ChipsReturned {
            player: PlayerId(2),
            amount: 2,
        }),
        event(TexasEvent::RunoutVoteStarted {
            current_player: PlayerId(0),
        }),
        ServerEvent::GameSnapshot(GameSnapshot::Texas(choosing)),
        event(TexasEvent::HandFinished { showdown: true }),
        ServerEvent::GameSnapshot(GameSnapshot::Texas(complete.clone())),
        ServerEvent::GameSnapshot(GameSnapshot::Texas(complete.clone())),
        event(TexasEvent::HandFinished { showdown: true }),
        ServerEvent::GameSnapshot(GameSnapshot::Texas(complete.clone())),
    ];
    // Includes a snapshot followed by a later event in one frame. Flattening that
    // frame's events against its snapshot incorrectly applies the later event early.
    let mut baseline = None;
    for cuts in 0..(1 << (messages.len() - 1)) {
        let mut updates = TexasChipUpdates::default();
        let mut state = TexasChipTableState::default();
        updates.apply(&ServerEvent::GameSnapshot(GameSnapshot::Texas(
            betting.clone(),
        )));
        flush(&mut updates, &mut state);
        for (index, message) in messages.iter().enumerate() {
            updates.apply(message);
            if cuts & (1 << index) != 0 {
                flush(&mut updates, &mut state);
                // Empty presentation frames may not consume pending events or restart flights.
                let chips = format!("{:?}", state.chips);
                flush(&mut updates, &mut state);
                assert_eq!(format!("{:?}", state.chips), chips);
            }
        }
        flush(&mut updates, &mut state);
        assert_balances(&state, &complete);
        assert!(state.settlement_applied);
        assert!(
            state
                .chips
                .iter()
                .any(|chip| chip.motion.is_some_and(|motion| motion.delay >= 0.86))
        );
        assert_eq!(state.event_serial, 4);
        let result = (state.event_serial, state.audio_cues.len());
        if let Some(expected) = &baseline {
            assert_eq!(&result, expected, "frame cuts: {cuts:b}");
        } else {
            baseline = Some(result);
        }
    }
}

#[test]
fn duplicate_betting_snapshots_preserve_action_chip_ids_and_motion_progress() {
    let mut game = choosing_snapshot();
    game.phase = TexasPhaseView::Betting {
        street: TexasStreet::PreFlop,
    };
    let mut state = TexasChipTableState::default();
    state.initialize(&game, true);
    game.players[2].stack -= 2;
    game.players[2].committed_street += 2;
    game.players[2].committed_total += 2;
    game.pot += 2;
    state.observe(
        &game,
        vec![TexasEvent::ActionApplied {
            player: PlayerId(2),
            action: TexasAction::RaiseTo(12),
            amount: 2,
            statistics: TexasActionStatistics {
                street: TexasStreet::PreFlop,
                all_in_amount: 0,
                full_raise: true,
                raised: true,
                bet_level: 1,
            },
        }],
        true,
    );
    assert_balances(&state, &game);
    assert!(state.chips.iter().any(|chip| chip.motion.is_some()));
    for chip in &mut state.chips {
        if let Some(motion) = chip.motion.as_mut() {
            motion.elapsed = 0.2;
        }
    }
    let chips = format!("{:?}", state.chips);
    state.audio_cues.clear();
    for _ in 0..3 {
        state.observe(&game, Vec::new(), true);
        assert_eq!(format!("{:?}", state.chips), chips);
        assert_balances(&state, &game);
        assert!(state.audio_cues.is_empty());
    }
}

#[test]
fn pending_events_wait_for_snapshot_and_are_discarded_on_exit() {
    let mut updates = TexasChipUpdates::default();
    let snapshot = ServerEvent::GameSnapshot(GameSnapshot::Texas(choosing_snapshot()));
    updates.apply(&snapshot);
    assert_eq!(updates.take().len(), 1);
    updates.apply(&event(TexasEvent::HandFinished { showdown: true }));
    for _ in 0..3 {
        assert!(updates.take().is_empty());
    }
    updates.apply(&ServerEvent::LeftRoom);
    updates.apply(&snapshot);
    let ready = updates.take();
    assert_eq!(ready.len(), 1);
    assert!(ready[0].1.is_empty());
}
