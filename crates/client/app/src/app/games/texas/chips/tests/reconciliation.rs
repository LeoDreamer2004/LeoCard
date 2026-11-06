use super::super::{ChipZone, TexasChipTableState};
use super::pot_test_snapshot;
use leocard_game_common::PlayerId;
use leocard_protocol::{TexasEvent, TexasPhaseView, TexasPotAwardView, TexasSnapshot};
use leocard_texas_holdem::TexasSquidTransfer;

pub(super) fn choosing_snapshot() -> TexasSnapshot {
    let mut game = pot_test_snapshot(&[(5, true, false), (10, true, false), (10, false, false)]);
    game.phase = TexasPhaseView::ChoosingRunout;
    game.squid.round = 1;
    game.squid.markers = vec![None];
    game
}

pub(super) fn complete_snapshot(two_runs: bool) -> TexasSnapshot {
    let mut game = choosing_snapshot();
    let awards = if two_runs {
        vec![(1, 8, 0), (2, 7, 1), (1, 5, 2), (2, 5, 1)]
    } else {
        vec![(1, 15, 0), (1, 10, 2)]
    };
    game.phase = TexasPhaseView::HandComplete {
        showdown: true,
        awards: awards
            .into_iter()
            .map(|(run, amount, winner)| TexasPotAwardView {
                run,
                amount,
                winners: vec![PlayerId(winner)],
                winning_category: None,
            })
            .collect(),
        table_winner: None,
        tournament_complete: false,
        reference_changes: Vec::new(),
    };
    game.squid.settled = true;
    game.squid.markers[0] = Some(PlayerId(2));
    game.squid.transfers = vec![TexasSquidTransfer {
        from: PlayerId(0),
        to: PlayerId(2),
        amount: 2,
    }];
    let stacks = if two_runs { [6, 12, 17] } else { [13, 0, 22] };
    for (player, stack) in game.players.iter_mut().zip(stacks) {
        player.stack = stack;
    }
    game.current_player = None;
    game
}

pub(super) fn assert_balances(state: &TexasChipTableState, game: &TexasSnapshot) {
    let complete = matches!(game.phase, TexasPhaseView::HandComplete { .. });
    let betting = matches!(game.phase, TexasPhaseView::Betting { .. });
    let value = |zone| {
        state
            .chips
            .iter()
            .filter(|chip| chip.zone == zone)
            .map(|chip| u32::from(chip.denomination))
            .sum::<u32>()
    };
    let mut bets = 0;
    for player in &game.players {
        assert_eq!(value(ChipZone::Stack(player.player.id)), player.stack);
        let bet = if betting { player.committed_street } else { 0 };
        assert_eq!(value(ChipZone::Bet(player.player.id)), bet);
        bets += bet;
    }
    let pot = state
        .chips
        .iter()
        .filter(|chip| matches!(chip.zone, ChipZone::Pot(_)))
        .map(|chip| u32::from(chip.denomination))
        .sum::<u32>();
    assert_eq!(pot, if complete { 0 } else { game.pot - bets });
    let total = state
        .chips
        .iter()
        .filter(|chip| chip.zone != ChipZone::Retired)
        .map(|chip| u32::from(chip.denomination))
        .sum::<u32>();
    assert_eq!(
        total,
        game.players.iter().map(|player| player.stack).sum::<u32>()
            + if complete { 0 } else { game.pot }
    );
}

fn advance_motions(state: &mut TexasChipTableState) {
    for chip in &mut state.chips {
        if let Some(motion) = chip.motion.as_mut() {
            motion.elapsed += 0.2;
        }
    }
}

#[test]
fn two_run_finish_before_snapshot_waits_then_animates_awards_and_squid_once() {
    for side_pots in [false, true] {
        let mut choosing = choosing_snapshot();
        // An earlier squid round's transfer data must not be used by an early finish.
        choosing.squid.transfers = vec![TexasSquidTransfer {
            from: PlayerId(2),
            to: PlayerId(0),
            amount: 3,
        }];
        let complete = complete_snapshot(true);
        let mut state = TexasChipTableState::default();
        state.initialize(&choosing, side_pots);
        state.observe(
            &choosing,
            vec![TexasEvent::HandFinished { showdown: true }],
            side_pots,
        );
        assert!(state.finish_observed);
        assert!(!state.settlement_applied);
        assert_balances(&state, &choosing);
        state.audio_cues.clear();
        for _ in 0..3 {
            state.observe(&choosing, Vec::new(), side_pots);
            assert_balances(&state, &choosing);
            assert!(!state.settlement_applied);
        }
        state.observe(&complete, Vec::new(), side_pots);
        assert_balances(&state, &complete);
        assert!(state.settlement_applied);
        // Reconciliation must not replace successful award/squid flights with static chips.
        assert!(
            state
                .chips
                .iter()
                .any(|chip| chip.motion.is_some_and(|motion| motion.delay >= 0.86))
        );
        advance_motions(&mut state);
        let chips = format!("{:?}", state.chips);
        let next_id = state.next_id;
        for events in [
            Vec::new(),
            vec![TexasEvent::HandFinished { showdown: true }],
            Vec::new(),
        ] {
            state.observe(&complete, events, side_pots);
            assert_balances(&state, &complete);
            assert_eq!(format!("{:?}", state.chips), chips);
            assert_eq!(state.next_id, next_id);
            assert!(state.audio_cues.is_empty());
        }
    }
}

#[test]
fn same_hand_refund_snapshot_without_events_repairs_only_changed_zones() {
    for side_pots in [false, true] {
        for choosing in [false, true] {
            let mut game =
                pot_test_snapshot(&[(12, false, false), (6, true, false), (4, false, true)]);
            let mut state = TexasChipTableState::default();
            state.initialize(&game, side_pots);
            let untouched = state
                .chips
                .iter()
                .filter(|chip| chip.zone == ChipZone::Stack(PlayerId(2)))
                .map(|chip| chip.id)
                .collect::<Vec<_>>();
            game.players[0].stack += 6;
            game.players[0].committed_total -= 6;
            game.players[0].committed_street -= 6;
            game.pot -= 6;
            if choosing {
                game.phase = TexasPhaseView::ChoosingRunout;
            }
            state.observe(&game, Vec::new(), side_pots);
            assert_balances(&state, &game);
            assert_eq!(
                state
                    .chips
                    .iter()
                    .filter(|chip| chip.zone == ChipZone::Stack(PlayerId(2)))
                    .map(|chip| chip.id)
                    .collect::<Vec<_>>(),
                untouched
            );
            // A changed side-pot boundary retains its normal divider feedback,
            // but repeated snapshots may not replay it or rebuild any chips.
            state.audio_cues.clear();
            advance_motions(&mut state);
            let chips = format!("{:?}", state.chips);
            state.observe(&game, Vec::new(), side_pots);
            assert_eq!(format!("{:?}", state.chips), chips);
            assert!(state.audio_cues.is_empty());
        }
    }
}

#[test]
fn reconnect_directly_complete_without_finish_never_replays_settlement() {
    for existing_hand in [false, true] {
        let mut state = TexasChipTableState::default();
        if existing_hand {
            state.initialize(&choosing_snapshot(), true);
        }
        let complete = complete_snapshot(true);
        state.observe(&complete, Vec::new(), true);
        assert!(state.settlement_applied);
        assert_balances(&state, &complete);
        assert!(state.chips.iter().all(|chip| chip.motion.is_none()));
        let chips = format!("{:?}", state.chips);
        for events in [
            Vec::new(),
            vec![TexasEvent::HandFinished { showdown: true }],
            Vec::new(),
        ] {
            state.observe(&complete, events, true);
            assert_balances(&state, &complete);
            assert_eq!(format!("{:?}", state.chips), chips);
            assert!(state.audio_cues.is_empty());
        }
    }
}

#[test]
fn ordinary_single_run_and_side_pot_settlement_keep_their_flights() {
    for side_pots in [false, true] {
        let mut state = TexasChipTableState::default();
        let mut betting = choosing_snapshot();
        betting.phase = TexasPhaseView::Betting {
            street: leocard_texas_holdem::TexasStreet::PreFlop,
        };
        state.initialize(&betting, side_pots);
        let complete = complete_snapshot(false);
        state.observe(
            &complete,
            vec![TexasEvent::HandFinished { showdown: true }],
            side_pots,
        );
        assert_balances(&state, &complete);
        assert!(
            state
                .chips
                .iter()
                .any(|chip| chip.motion.is_some_and(|motion| motion.delay >= 0.86))
        );
        advance_motions(&mut state);
        let chips = format!("{:?}", state.chips);
        state.observe(&complete, Vec::new(), side_pots);
        assert_eq!(format!("{:?}", state.chips), chips);
    }
}
