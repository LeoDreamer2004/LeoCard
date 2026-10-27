use super::DoudizhuUiState;
use leocard_doudizhu::Card;
use leocard_protocol::MatchId;

#[test]
fn initial_and_redealt_hands_animate_once_without_restart_on_refresh() {
    let match_id = MatchId([11; 16]);
    let cards = [Card(0), Card(4), Card(8)];
    let mut state = DoudizhuUiState::default();
    state.pending_deal = Some(match_id);
    let initial = state.observe_dealt_hand(match_id, &cards);
    assert_eq!(initial.len(), cards.len());
    assert!(initial.windows(2).all(|pair| pair[0].1 < pair[1].1));
    state
        .card_animations
        .get_mut(&Card(0))
        .unwrap()
        .deal_elapsed += 0.05;
    let elapsed = state.card_animations[&Card(0)].deal_elapsed;
    assert!(state.observe_dealt_hand(match_id, &cards).is_empty());
    assert_eq!(state.card_animations[&Card(0)].deal_elapsed, elapsed);

    state.selected.insert(Card(0));
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
    state.observe_dealt_hand(match_id, &[Card(0), Card(4)]);
    let initial_remaining = state
        .card_animations
        .values()
        .map(|animation| 0.20 - animation.deal_elapsed)
        .fold(0.0, f32::max);
    let kitty = state.observe_dealt_hand(match_id, &[Card(0), Card(4), Card(8)]);
    assert_eq!(kitty, vec![(Card(8), initial_remaining)]);
    assert!(
        state
            .observe_dealt_hand(match_id, &[Card(0), Card(4), Card(8)])
            .is_empty()
    );
}

#[test]
fn joining_an_existing_hand_does_not_replay_its_deal() {
    let mut state = DoudizhuUiState::default();
    let cards = [Card(0), Card(4)];
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
