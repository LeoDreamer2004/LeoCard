use super::*;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, ShengjiEvent, ShengjiPublicPlay};
use leocard_shengji::ShengjiCard;
use leocard_shengji::{Category, Component, ShengjiClassifiedPlay, ShengjiRank};
use leocard_shengji::{ShengjiSuit, ShengjiThrowPenalty, ShengjiTrump};

#[test]
fn five_and_ten_point_throw_penalties_use_distinct_sound_weights() {
    let mut five = Vec::new();
    queue_throw_failure_audio(&mut five, ShengjiThrowPenalty::FivePerCard);
    let mut ten = Vec::new();
    queue_throw_failure_audio(&mut ten, ShengjiThrowPenalty::TenPerCard);

    assert!(
        five.iter()
            .any(|cue| cue.kind == ShengjiSoundKind::PenaltyFive)
    );
    assert!(!five.iter().any(|cue| cue.kind == ShengjiSoundKind::Heavy));
    assert!(
        ten.iter()
            .any(|cue| cue.kind == ShengjiSoundKind::PenaltyTen)
    );
    assert!(ten.iter().any(|cue| cue.kind == ShengjiSoundKind::Heavy));
}

#[test]
fn presentations_queue_instead_of_overwriting_a_rule_effect() {
    let mut state = ShengjiPresentationState::default();
    assert_eq!(
        state.activate(
            ShengjiPresentationKind::PowerOutage {
                from_dealer: Some(PlayerId(0)),
                dealer: PlayerId(1),
                level: ShengjiRank::Three,
            },
            1.2,
        ),
        0.0
    );
    assert_eq!(
        state.activate(
            ShengjiPresentationKind::BottomCopy {
                cards: Vec::new(),
                from_player: Some(PlayerId(0)),
                player: PlayerId(1),
            },
            0.7,
        ),
        1.2
    );
    assert!(matches!(
        state.active.as_ref().map(|active| &active.kind),
        Some(ShengjiPresentationKind::PowerOutage { .. })
    ));
    assert!(matches!(
        state.queued.front().map(|active| &active.kind),
        Some(ShengjiPresentationKind::BottomCopy { .. })
    ));
}

#[test]
fn only_a_structure_matching_winning_trump_play_triggers_the_target_effect() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let single = |player: u8, card: ShengjiCard, category| ShengjiPublicPlay {
        player: PlayerId(player),
        play: ShengjiClassifiedPlay {
            cards: vec![card],
            category,
            components: vec![Component::Single {
                card,
                strength: trump.strength(card),
            }],
        },
        throw_penalty: 0,
    };
    let lead = single(
        0,
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace),
        Category::Suit(ShengjiSuit::Spade),
    );
    let first_kill = single(
        1,
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        Category::Trump,
    );
    let cover_kill = single(
        2,
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ace),
        Category::Trump,
    );
    let losing_trump = single(
        3,
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Four),
        Category::Trump,
    );

    let mut state = ShengjiPresentationState::default();
    state.current_trick_plays.push(lead);
    assert_eq!(
        state.winning_trump_kill(&first_kill, false, Some(trump)),
        Some(false)
    );
    state.current_trick_plays.push(first_kill);
    assert_eq!(
        state.winning_trump_kill(&cover_kill, false, Some(trump)),
        Some(true)
    );
    state.current_trick_plays.push(cover_kill);
    assert_eq!(
        state.winning_trump_kill(&losing_trump, false, Some(trump)),
        None
    );

    let pair_cards = [
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Nine),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Nine),
    ];
    let pair_lead = ShengjiPublicPlay {
        player: PlayerId(0),
        play: ShengjiClassifiedPlay {
            cards: pair_cards.to_vec(),
            category: Category::Suit(ShengjiSuit::Spade),
            components: vec![Component::Pair {
                cards: pair_cards,
                strength: trump.strength(pair_cards[0]),
            }],
        },
        throw_penalty: 0,
    };
    let discard_cards = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
    ];
    let structure_mismatch = ShengjiPublicPlay {
        player: PlayerId(1),
        play: ShengjiClassifiedPlay {
            cards: discard_cards.to_vec(),
            category: Category::Trump,
            components: discard_cards
                .into_iter()
                .map(|card| Component::Single {
                    card,
                    strength: trump.strength(card),
                })
                .collect(),
        },
        throw_penalty: 0,
    };
    state.current_trick_plays = vec![pair_lead];
    assert_eq!(
        state.winning_trump_kill(&structure_mismatch, false, Some(trump)),
        None
    );
}

#[test]
fn previous_trick_becomes_replayable_only_after_all_four_public_plays_finish() {
    let mut state = ShengjiPresentationState::default();
    for player in 0..4 {
        let card = ShengjiCard::suited(player, ShengjiSuit::Spade, ShengjiRank::Three);
        state.observe_trick_event(&ShengjiEvent::CardsPlayed {
            play: ShengjiPublicPlay {
                player: PlayerId(player),
                play: ShengjiClassifiedPlay {
                    cards: vec![card],
                    category: Category::Suit(ShengjiSuit::Spade),
                    components: vec![Component::Single { card, strength: 1 }],
                },
                throw_penalty: 0,
            },
            is_lead: player == 0,
        });
        assert!(!state.has_previous_trick());
    }
    state.observe_trick_event(&ShengjiEvent::TrickFinished {
        winner: PlayerId(0),
        points: 0,
        collecting_score: 0,
    });

    assert!(state.has_previous_trick());
    assert!(state.revealed_previous_trick().is_none());
    state.reveal_previous_trick();
    assert_eq!(state.revealed_previous_trick().unwrap().len(), 4);
}
