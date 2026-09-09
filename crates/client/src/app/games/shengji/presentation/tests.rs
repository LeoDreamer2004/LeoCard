use super::routes::shengji_seat_route_anchor;
use super::*;
use leocard_protocol::ShengjiEvent;
use leocard_protocol::{PlayerId, ShengjiPublicPlay};
use leocard_shengji::{Category, Component, ShengjiBidTrump, ShengjiClassifiedPlay, ShengjiRank};
use leocard_shengji::{ShengjiCard, ShengjiSuit, ShengjiThrowPenalty, ShengjiTrump};

#[test]
fn routed_presentations_point_at_all_four_relative_seats() {
    let own = shengji_seat_route_anchor(0);
    let left = shengji_seat_route_anchor(1);
    let top = shengji_seat_route_anchor(2);
    let right = shengji_seat_route_anchor(3);

    assert!(own.y > 50.0);
    assert!(top.y < 50.0);
    assert!(left.x < 50.0);
    assert!(right.x > 50.0);
    assert_ne!(own, top);
    assert_ne!(left, right);
}

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
fn every_shengji_structure_maps_to_the_declared_presentation_family() {
    let copies = |rank, count: u8| {
        (0..count)
            .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Spade, rank))
            .collect::<Vec<_>>()
    };
    let play = |components: Vec<Component>| ShengjiClassifiedPlay {
        cards: components.iter().flat_map(Component::cards).collect(),
        category: Category::Suit(ShengjiSuit::Spade),
        components,
    };
    let single_card = copies(ShengjiRank::Three, 1)[0];
    let pair_cards: [ShengjiCard; 2] = copies(ShengjiRank::Four, 2).try_into().unwrap();
    let triple_cards: [ShengjiCard; 3] = copies(ShengjiRank::Five, 3).try_into().unwrap();
    let bomb_cards: [ShengjiCard; 4] = copies(ShengjiRank::Six, 4).try_into().unwrap();
    let tractor_cards = [copies(ShengjiRank::Seven, 2), copies(ShengjiRank::Eight, 2)].concat();
    let titanic_cards = [copies(ShengjiRank::Nine, 3), copies(ShengjiRank::Ten, 3)].concat();
    let spaceship_cards = [copies(ShengjiRank::Jack, 4), copies(ShengjiRank::Queen, 4)].concat();

    let cases = [
        (
            play(vec![Component::Single {
                card: single_card,
                strength: 1,
            }]),
            ShengjiPlayPresentationKind::Single,
        ),
        (
            play(vec![Component::Pair {
                cards: pair_cards,
                strength: 2,
            }]),
            ShengjiPlayPresentationKind::Pair,
        ),
        (
            play(vec![Component::Tractor {
                cards: tractor_cards,
                pair_count: 2,
                top_strength: 4,
            }]),
            ShengjiPlayPresentationKind::Tractor,
        ),
        (
            play(vec![Component::Triple {
                cards: triple_cards,
                strength: 5,
            }]),
            ShengjiPlayPresentationKind::Triple,
        ),
        (
            play(vec![Component::Titanic {
                cards: titanic_cards,
                triple_count: 2,
                top_strength: 7,
            }]),
            ShengjiPlayPresentationKind::Titanic,
        ),
        (
            play(vec![Component::Quad {
                cards: bomb_cards,
                strength: 8,
            }]),
            ShengjiPlayPresentationKind::Bomb,
        ),
        (
            play(vec![Component::Spaceship {
                cards: spaceship_cards,
                quad_count: 2,
                top_strength: 10,
            }]),
            ShengjiPlayPresentationKind::Spaceship,
        ),
        (
            play(vec![
                Component::Single {
                    card: single_card,
                    strength: 1,
                },
                Component::Pair {
                    cards: pair_cards,
                    strength: 2,
                },
            ]),
            ShengjiPlayPresentationKind::Throw,
        ),
    ];
    for (classified, expected) in cases {
        assert_eq!(classify_play_presentation(&classified), expected);
    }
}

#[test]
fn rare_play_effects_last_longer_than_routine_plays() {
    assert!(
        ShengjiPlayPresentationKind::Spaceship.duration()
            > ShengjiPlayPresentationKind::Pair.duration()
    );
    assert!(
        ShengjiPlayPresentationKind::Titanic.duration()
            > ShengjiPlayPresentationKind::Triple.duration()
    );
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
            ShengjiPresentationKind::Declaration {
                player: PlayerId(1),
                trump: ShengjiBidTrump::Suit(ShengjiSuit::Spade),
                label: "亮主",
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
        Some(ShengjiPresentationKind::Declaration { .. })
    ));
}

#[test]
fn routine_cards_and_following_mixed_shapes_do_not_show_type_labels() {
    assert!(!should_show_play_presentation(
        ShengjiPlayPresentationKind::Single,
        true,
        0,
    ));
    assert!(!should_show_play_presentation(
        ShengjiPlayPresentationKind::Pair,
        true,
        0,
    ));
    assert!(!should_show_play_presentation(
        ShengjiPlayPresentationKind::Triple,
        true,
        0,
    ));
    assert!(!should_show_play_presentation(
        ShengjiPlayPresentationKind::Throw,
        false,
        0,
    ));
    assert!(should_show_play_presentation(
        ShengjiPlayPresentationKind::Throw,
        true,
        0,
    ));
    assert!(should_show_play_presentation(
        ShengjiPlayPresentationKind::Tractor,
        false,
        0,
    ));
    assert!(!should_show_play_presentation(
        ShengjiPlayPresentationKind::Throw,
        true,
        5,
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
    assert_eq!(state.previous_trick_reveal_remaining, 2.0);
    assert_eq!(state.revealed_previous_trick().unwrap().len(), 4);
}
