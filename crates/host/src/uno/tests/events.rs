use super::*;

#[test]
fn identical_pair_broadcasts_both_card_play_events_in_order() {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let second = UnoCard::number(UnoColor::Red, 7, 1);
    let events = events_for_outcome(
        &ActionOutcome::Played {
            player: UnoPlayerId(0),
            card: second,
            next_player: UnoPlayerId(1),
            effect: None,
        },
        &[(first, None), (second, None)],
    );
    assert_eq!(
        events,
        vec![
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: first,
                chosen_color: None,
                play_index: 0,
                play_count: 2,
            },
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: second,
                chosen_color: None,
                play_index: 1,
                play_count: 2,
            },
        ]
    );
}

#[test]
fn stack_number_broadcasts_every_revealed_card_after_the_play() {
    let played = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let skipped = UnoCard::action(UnoColor::Blue, UnoFace::Skip, 0);
    let number = UnoCard::number(UnoColor::Yellow, 6, 0);
    let events = events_for_outcome(
        &ActionOutcome::Played {
            player: UnoPlayerId(0),
            card: played,
            next_player: UnoPlayerId(1),
            effect: Some(PlayedEffect::StackNumberRevealed {
                cards: vec![skipped, number],
                value: 6,
            }),
        },
        &[(played, Some(UnoColor::Red))],
    );
    assert_eq!(
        events,
        vec![
            UnoEvent::CardPlayed {
                player: PlayerId(0),
                card: played,
                chosen_color: Some(UnoColor::Red),
                play_index: 0,
                play_count: 1,
            },
            UnoEvent::StackNumberRevealed {
                player: PlayerId(0),
                cards: vec![skipped, number],
                value: 6,
            },
        ]
    );
}

#[test]
fn random_flip_pairing_remains_valid_and_public_faces_keep_unique_ids() {
    let rules = UnoRuleSet {
        mode: Mode::Flip,
        flip: FlipRuleSet {
            random_pairing: true,
            ..FlipRuleSet::default()
        },
        ..UnoRuleSet::default()
    };
    let deck = shuffled_uno_deck(rules);
    validate_deck(&deck, rules).unwrap();
    assert_eq!(
        deck.iter()
            .map(|card| card.public_face())
            .collect::<HashSet<_>>()
            .len(),
        112
    );
    assert_eq!(
        deck.iter()
            .filter_map(|card| card.opposite_public_face())
            .collect::<HashSet<_>>()
            .len(),
        112
    );
}
