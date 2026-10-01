use super::denominations::{change_for, initial_chip_denominations};
use super::ledger::visual_pots;
use super::*;
use crate::app::games::texas_holdem::TexasSoundKind;
use bevy::prelude::*;
use leocard_protocol::{MatchId, PlayerId, SeatId};
#[cfg(test)]
use leocard_protocol::{PlayerGameProfiles, ProfileId};
use leocard_protocol::{
    TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemSnapshot,
};
use leocard_texas_holdem::{TexasHoldemAction, TexasHoldemActionStatistics, TexasHoldemStreet};

fn pot_test_snapshot(committed: &[(u32, bool, bool)]) -> TexasHoldemSnapshot {
    let players = committed
        .iter()
        .enumerate()
        .map(
            |(index, &(amount, all_in, folded))| TexasHoldemPlayerState {
                id: PlayerId(index as u8),
                profile_id: ProfileId([index as u8; 32]),
                name: format!("P{index}"),
                avatar: None,
                seat: SeatId(index as u8),
                stack: if all_in {
                    0
                } else {
                    20_u32.saturating_sub(amount)
                },
                hand_start_stack: 20,
                committed_street: amount,
                committed_total: amount,
                folded,
                all_in,
                connected: true,
                auto_play: false,
                ready: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            },
        )
        .collect::<Vec<_>>();
    TexasHoldemSnapshot {
        match_id: MatchId([1; 16]),
        hand_number: 1,
        host_port: 5230,
        you: PlayerId(0),
        host: PlayerId(0),
        players,
        your_hole_cards: Vec::new(),
        revealed_hands: Vec::new(),
        community: Vec::new(),
        draw_pile_len: 52,
        dealer: PlayerId(0),
        small_blind: PlayerId(1),
        big_blind: PlayerId(2),
        current_player: Some(PlayerId(0)),
        blind_to_post: None,
        current_bet: committed.iter().map(|entry| entry.0).max().unwrap_or(0),
        minimum_raise_to: 1,
        amount_to_call: 0,
        raise_allowed: true,
        pot: committed.iter().map(|entry| entry.0).sum(),
        phase: TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::PreFlop,
        },
    }
}

#[test]
fn initial_distributions_keep_five_single_chips_and_exact_value() {
    let expected = [
        (5, (0, 0, 5)),
        (10, (0, 1, 5)),
        (20, (0, 3, 5)),
        (30, (1, 3, 5)),
        (40, (2, 3, 5)),
        (50, (3, 3, 5)),
    ];
    for (total, (tens, fives, ones)) in expected {
        let chips = initial_chip_denominations(total);
        assert_eq!(
            chips.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
            total
        );
        assert_eq!(chips.iter().filter(|chip| **chip == 10).count(), tens);
        assert_eq!(chips.iter().filter(|chip| **chip == 5).count(), fives);
        assert_eq!(chips.iter().filter(|chip| **chip == 1).count(), ones);
    }
}

#[test]
fn every_supported_chip_can_be_changed_exactly() {
    for denomination in [5, 10, 25, 100] {
        let change = change_for(denomination);
        assert_eq!(
            change.iter().map(|chip| u32::from(*chip)).sum::<u32>(),
            u32::from(denomination)
        );
        assert!(change.iter().all(|chip| *chip < denomination));
    }
}

#[test]
fn side_pots_only_split_at_all_in_caps_and_keep_correct_eligibility() {
    let no_all_in = visual_pots(&pot_test_snapshot(&[
        (5, false, false),
        (10, false, false),
        (20, false, false),
    ]));
    assert_eq!(no_all_in.len(), 1);
    assert_eq!(no_all_in[0].amount, 35);

    let split = visual_pots(&pot_test_snapshot(&[
        (5, true, false),
        (10, true, false),
        (20, false, false),
    ]));
    assert_eq!(
        split.iter().map(|pot| pot.amount).collect::<Vec<_>>(),
        vec![15, 10, 10]
    );
    assert_eq!(
        split[0].eligible,
        vec![PlayerId(0), PlayerId(1), PlayerId(2)]
    );
    assert_eq!(split[1].eligible, vec![PlayerId(1), PlayerId(2)]);
    assert_eq!(split[2].eligible, vec![PlayerId(2)]);
}

#[test]
fn authoritative_action_audio_is_not_requeued_by_an_empty_ui_refresh() {
    let snapshot = pot_test_snapshot(&[(0, false, false), (0, false, false), (0, false, false)]);
    let mut state = TexasChipTableState::default();
    state.initialize(&snapshot);
    state.observe(
        &snapshot,
        vec![TexasHoldemEvent::ActionApplied {
            player: PlayerId(1),
            action: TexasHoldemAction::Check,
            amount: 0,
            statistics: TexasHoldemActionStatistics {
                street: TexasHoldemStreet::PreFlop,
                all_in_amount: 0,
                full_raise: false,
                raised: false,
                bet_level: 1,
            },
        }],
    );
    assert_eq!(state.audio_cues.len(), 1);
    assert_eq!(state.audio_cues[0].kind, TexasSoundKind::Check);

    state.audio_cues.clear();
    state.observe(&snapshot, Vec::new());
    assert!(state.audio_cues.is_empty());
}
