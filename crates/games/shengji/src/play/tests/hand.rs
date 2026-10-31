use super::fixtures::{pair, quad, triple, trump};
use crate::{ShengjiComponent, ShengjiRank, ShengjiSuit, ShengjiTrump, hand_sequences};

#[test]
fn a_triple_can_supply_a_pair_inside_a_longer_tractor() {
    let hand = [
        pair(ShengjiSuit::Club, ShengjiRank::Eight).to_vec(),
        triple(ShengjiSuit::Club, ShengjiRank::Seven).to_vec(),
        pair(ShengjiSuit::Club, ShengjiRank::Six).to_vec(),
    ]
    .concat();
    let sequences = hand_sequences(&hand, trump()).unwrap();
    assert!(matches!(
        sequences.as_slice(),
        [ShengjiComponent::Tractor { pair_count: 3, .. }]
    ));
}

#[test]
fn larger_runs_are_not_repeated_as_smaller_hints() {
    let hand = [
        quad(ShengjiSuit::Club, ShengjiRank::Eight).to_vec(),
        quad(ShengjiSuit::Club, ShengjiRank::Seven).to_vec(),
        triple(ShengjiSuit::Club, ShengjiRank::Five).to_vec(),
        triple(ShengjiSuit::Club, ShengjiRank::Four).to_vec(),
        pair(ShengjiSuit::Club, ShengjiRank::Ace).to_vec(),
        pair(ShengjiSuit::Club, ShengjiRank::King).to_vec(),
    ]
    .concat();
    let sequences = hand_sequences(&hand, trump()).unwrap();
    assert!(matches!(
        sequences.as_slice(),
        [
            ShengjiComponent::Spaceship { quad_count: 2, .. },
            ShengjiComponent::Titanic {
                triple_count: 2,
                ..
            },
            ShengjiComponent::Tractor { pair_count: 2, .. },
        ]
    ));
}

#[test]
fn sequences_skip_the_level_rank_and_do_not_merge_different_suits() {
    let trump = ShengjiTrump::new(ShengjiRank::Eight, Some(ShengjiSuit::Heart)).unwrap();
    let hand = [
        pair(ShengjiSuit::Club, ShengjiRank::Nine),
        pair(ShengjiSuit::Club, ShengjiRank::Seven),
        pair(ShengjiSuit::Diamond, ShengjiRank::Six),
        pair(ShengjiSuit::Diamond, ShengjiRank::Eight),
        pair(ShengjiSuit::Spade, ShengjiRank::Eight),
    ]
    .concat();
    let sequences = hand_sequences(&hand, trump).unwrap();
    assert!(matches!(
        sequences.as_slice(),
        [ShengjiComponent::Tractor { pair_count: 2, .. }]
    ));
    assert!(
        sequences[0]
            .cards()
            .iter()
            .all(|card| card.suit() == Some(ShengjiSuit::Club))
    );
}
