use super::DoudizhuUiState;
use super::history::DoudizhuTrickHistory;
use leocard_doudizhu::{DoudizhuAction, DoudizhuCard};
use leocard_game_common::PlayerId;
use leocard_protocol::MatchId;

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
