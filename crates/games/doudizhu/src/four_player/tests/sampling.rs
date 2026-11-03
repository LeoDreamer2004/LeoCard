use crate::{
    DoudizhuAction, DoudizhuCard, DoudizhuGameState, DoudizhuMode, DoudizhuRuleSet, FourPlayerBot,
    FourPlayerObservation, build_deck,
};
use leocard_game_common::{PlayerId, SearchRandom};
use std::collections::HashSet;

#[test]
fn hidden_hands_and_deck_order_do_not_enter_observation_or_fallback() {
    let deck = build_deck(DoudizhuMode::Four);
    let mut swapped = deck.clone();
    swapped.swap(4, 10); // Different ranks held by landlord and farmer 2, never observer 1 or kitty.
    let mut observations = Vec::new();
    for deck in [deck, swapped] {
        let mut game = DoudizhuGameState::new(
            DoudizhuRuleSet::for_mode(DoudizhuMode::Four),
            deck,
            PlayerId(0),
        )
        .unwrap();
        game.act(PlayerId(0), DoudizhuAction::Bid(3)).unwrap();
        game.act(PlayerId(0), DoudizhuAction::Play(vec![DoudizhuCard(0)]))
            .unwrap();
        observations.push(FourPlayerObservation::from_game(&game).unwrap());
    }
    assert_eq!(observations[0], observations[1]);
    assert_eq!(
        FourPlayerBot::fallback(&observations[0]),
        FourPlayerBot::fallback(&observations[1])
    );
    let mut different = false;
    let mut previous = None;
    for seed in 0..12 {
        let sample = observations[0].sample(&mut SearchRandom::new(seed));
        assert_eq!(sample.hand(PlayerId(1)).unwrap(), observations[0].hand);
        assert_eq!(
            sample.hand_lengths(),
            observations[0]
                .counts
                .iter()
                .map(|count| *count as u16)
                .collect::<Vec<_>>()
        );
        for card in &observations[0].landlord_known {
            assert!(sample.hand(PlayerId(0)).unwrap().contains(card));
        }
        let cards: HashSet<_> = (0..4)
            .flat_map(|player| sample.hand(PlayerId(player)).unwrap())
            .collect();
        assert_eq!(cards.len(), 107);
        assert!(!cards.contains(&DoudizhuCard(0)));
        let landlord_hand = sample.hand(PlayerId(0)).unwrap().to_vec();
        if previous.as_ref().is_some_and(|hand| *hand != landlord_hand) {
            different = true;
        }
        previous = Some(landlord_hand);
    }
    assert!(different, "samples must not recover one authoritative deal");
}
