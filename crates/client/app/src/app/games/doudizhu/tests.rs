use super::DoudizhuUiState;
use super::hand_interaction::{refine_drag_selection, retain_largest_straight};
use super::history::DoudizhuTrickHistory;
use leocard_doudizhu::{
    DoudizhuAction, DoudizhuCard, DoudizhuMode, DoudizhuPlayKind, DoudizhuRuleSet, classify,
};
use leocard_game_common::PlayerId;
use leocard_protocol::MatchId;
use std::collections::HashSet;

#[test]
fn drag_selection_preserves_consecutive_pairs_in_both_modes() {
    for (mode, second_card_offset) in [(DoudizhuMode::Three, 1), (DoudizhuMode::Four, 54)] {
        let rules = DoudizhuRuleSet::for_mode(mode);
        let mut selected: HashSet<_> = (0..5)
            .flat_map(|rank| {
                [
                    DoudizhuCard(rank * 4),
                    DoudizhuCard(second_card_offset + rank * 4),
                ]
            })
            .collect();
        let original = selected.clone();
        refine_drag_selection(&mut selected, rules);
        assert_eq!(selected, original);
        assert_eq!(
            classify(&selected.iter().copied().collect::<Vec<_>>(), rules).map(|play| play.kind),
            Some(DoudizhuPlayKind::Pairs)
        );
    }
}

#[test]
fn drag_selection_preserves_airplane_instead_of_reducing_it_to_a_straight() {
    let rules = DoudizhuRuleSet::for_mode(DoudizhuMode::Three);
    let mut selected: HashSet<_> = (0..5)
        .flat_map(|rank| (0..3).map(move |suit| DoudizhuCard(rank * 4 + suit)))
        .collect();
    let original = selected.clone();
    refine_drag_selection(&mut selected, rules);
    assert_eq!(selected, original);
    assert_eq!(
        classify(&selected.iter().copied().collect::<Vec<_>>(), rules).map(|play| play.kind),
        Some(DoudizhuPlayKind::Airplane)
    );
}

#[test]
fn largest_straight_prefers_length_then_highest_rank() {
    let mut selected: HashSet<_> = (0..=5)
        .chain(7..=11)
        .map(|rank| DoudizhuCard(rank * 4))
        .collect();
    retain_largest_straight(&mut selected);
    assert_eq!(
        selected,
        (0..=5).map(|rank| DoudizhuCard(rank * 4)).collect()
    );

    let mut selected: HashSet<_> = (0..=4)
        .chain(6..=10)
        .map(|rank| DoudizhuCard(rank * 4))
        .collect();
    retain_largest_straight(&mut selected);
    assert_eq!(
        selected,
        (6..=10).map(|rank| DoudizhuCard(rank * 4)).collect()
    );
}

#[test]
fn largest_straight_discards_duplicate_ranks_and_cards_above_ace() {
    let rules = DoudizhuRuleSet::for_mode(DoudizhuMode::Four);
    let mut selected: HashSet<_> = (3..=7)
        .map(|rank| DoudizhuCard(rank * 4))
        .chain([DoudizhuCard(54 + 5 * 4), DoudizhuCard(48), DoudizhuCard(52)])
        .collect();
    refine_drag_selection(&mut selected, rules);
    assert_eq!(
        selected,
        (3..=7).map(|rank| DoudizhuCard(rank * 4)).collect()
    );
    let play = classify(&selected.iter().copied().collect::<Vec<_>>(), rules)
        .expect("selected cards form a valid straight");
    assert_eq!(play.kind, DoudizhuPlayKind::Straight);
}

#[test]
fn largest_straight_preserves_cards_without_a_straight() {
    let mut selected: HashSet<_> = [9, 10, 11, 12, 13]
        .into_iter()
        .map(|rank| DoudizhuCard(rank * 4))
        .collect();
    let original = selected.clone();
    retain_largest_straight(&mut selected);
    assert_eq!(selected, original);
}

#[test]
fn initial_and_redealt_hands_animate_once_without_restart_on_refresh() {
    let match_id = MatchId([11; 16]);
    let cards = [DoudizhuCard(0), DoudizhuCard(4), DoudizhuCard(8)];
    let mut state = DoudizhuUiState::default();
    state.pending_deal = Some(match_id);
    let initial = state.observe_dealt_hand(match_id, &cards);
    assert_eq!(initial.len(), cards.len());
    assert!(initial.windows(2).all(|pair| pair[0].1 < pair[1].1));
    state
        .card_animations
        .get_mut(&DoudizhuCard(0))
        .unwrap()
        .deal_elapsed += 0.05;
    let elapsed = state.card_animations[&DoudizhuCard(0)].deal_elapsed;
    assert!(state.observe_dealt_hand(match_id, &cards).is_empty());
    assert_eq!(
        state.card_animations[&DoudizhuCard(0)].deal_elapsed,
        elapsed
    );

    state.selected.insert(DoudizhuCard(0));
    state.pending_deal = Some(match_id);
    assert_eq!(
        state.observe_dealt_hand(match_id, &cards).len(),
        cards.len()
    );
    assert!(state.selected.is_empty());
    assert!(state.observe_dealt_hand(match_id, &cards).is_empty());
}

#[test]
fn landlord_kitty_cards_animate_after_the_pending_initial_hand() {
    let match_id = MatchId([12; 16]);
    let mut state = DoudizhuUiState::default();
    state.pending_deal = Some(match_id);
    state.observe_dealt_hand(match_id, &[DoudizhuCard(0), DoudizhuCard(4)]);
    let initial_remaining = state
        .card_animations
        .values()
        .map(|animation| 0.20 - animation.deal_elapsed)
        .fold(0.0, f32::max);
    let kitty = state.observe_dealt_hand(
        match_id,
        &[DoudizhuCard(0), DoudizhuCard(4), DoudizhuCard(8)],
    );
    assert_eq!(kitty, vec![(DoudizhuCard(8), initial_remaining)]);
    assert!(
        state
            .observe_dealt_hand(
                match_id,
                &[DoudizhuCard(0), DoudizhuCard(4), DoudizhuCard(8)]
            )
            .is_empty()
    );
}

#[test]
fn joining_an_existing_hand_does_not_replay_its_deal() {
    let mut state = DoudizhuUiState::default();
    let cards = [DoudizhuCard(0), DoudizhuCard(4)];
    assert!(
        state
            .observe_dealt_hand(MatchId([13; 16]), &cards)
            .is_empty()
    );
    assert!(
        state
            .card_animations
            .values()
            .all(|animation| !animation.dealing)
    );
    assert!(
        state
            .observe_dealt_hand(MatchId([13; 16]), &cards)
            .is_empty()
    );
}

#[test]
fn previous_trick_waits_for_every_pass_and_a_raise_resets_the_pass_count() {
    for count in [3, 4] {
        let mut history = DoudizhuTrickHistory::default();
        history.observe(PlayerId(0), &DoudizhuAction::Bid(3), count);
        history.observe(
            PlayerId(0),
            &DoudizhuAction::Play(vec![DoudizhuCard(0)]),
            count,
        );
        history.observe(PlayerId(1), &DoudizhuAction::Pass, count);
        let winning_play = DoudizhuAction::Play(vec![DoudizhuCard(4)]);
        history.observe(PlayerId(2), &winning_play, count);
        for offset in 1..count {
            assert!(!history.has_previous());
            history.observe(PlayerId((2 + offset) % count), &DoudizhuAction::Pass, count);
        }
        assert!(history.has_previous());
        assert!(history.revealed_actions().is_none());
        history.reveal();
        let actions = history.revealed_actions().unwrap().to_vec();
        assert_eq!(actions.len(), usize::from(count));
        assert_eq!(actions[2], Some(winning_play));
        assert!(
            actions.iter().enumerate().all(|(index, action)| {
                index == 2 || matches!(action, Some(DoudizhuAction::Pass))
            })
        );

        // New plays and ordinary UI refreshes don't overwrite the recalled trick.
        history.observe(
            PlayerId(2),
            &DoudizhuAction::Play(vec![DoudizhuCard(8)]),
            count,
        );
        assert!(!history.advance(1.0));
        assert_eq!(history.revealed_actions().unwrap(), actions);
        assert!(history.advance(1.0));
        assert!(history.revealed_actions().is_none());
        history.reveal();
        assert!(history.revealed_actions().is_some());
        history.clear();
        assert!(!history.has_previous());
        assert!(history.revealed_actions().is_none());
    }
}

#[test]
fn missed_lead_does_not_create_a_pass_only_previous_trick() {
    let mut history = DoudizhuTrickHistory::default();
    history.observe(PlayerId(1), &DoudizhuAction::Pass, 3);
    history.observe(PlayerId(2), &DoudizhuAction::Pass, 3);
    assert!(!history.has_previous());
    history.reveal();
    assert!(history.revealed_actions().is_none());
}
