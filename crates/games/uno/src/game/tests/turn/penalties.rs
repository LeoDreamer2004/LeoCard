use super::super::prelude::*;

#[test]
fn action_stacking_controls_draw_two_and_draw_four_chains() {
    let p0_draw_two = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let p1_draw_two = card(UnoColor::Yellow, UnoFace::DrawTwo, 0);
    let p2_draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    // Round-robin dealing positions are 0..=5; put the required cards explicitly.
    let mut deck = build_deck();
    for (position, required) in [
        (0, p0_draw_two),
        (1, p1_draw_two),
        (2, p2_draw_four),
        (42, start),
    ] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck.clone()).unwrap();
    game.play_card(UnoPlayerId(0), p0_draw_two, None).unwrap();
    assert_eq!(
        game.play_card(UnoPlayerId(1), p1_draw_two, None),
        Err(GameError::CannotStack(p1_draw_two))
    );

    let mut game = GameState::new_with_deck(
        UnoRuleSet {
            action_stacking: true,
            ..UnoRuleSet::default()
        },
        6,
        deck,
    )
    .unwrap();
    game.play_card(UnoPlayerId(0), p0_draw_two, None).unwrap();
    game.play_card(UnoPlayerId(1), p1_draw_two, None).unwrap();
    game.play_card(UnoPlayerId(2), p2_draw_four, Some(UnoColor::Blue))
        .unwrap();
    assert_eq!(game.turn().unwrap().pending_draw, 8);
}

#[test]
fn draw_four_after_draw_two_only_considers_matching_draw_two_for_challenge() {
    let draw_two = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let same_color_number = card(UnoColor::Red, UnoFace::Number(7), 0);
    let same_color_draw_two = card(UnoColor::Red, UnoFace::DrawTwo, 1);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);

    for alternative in [same_color_number, same_color_draw_two] {
        let mut deck = build_deck();
        for (position, required) in [(0, draw_two), (1, draw_four), (7, alternative), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        if alternative == same_color_number {
            for position in [1, 13, 19, 25, 31, 37] {
                if deck[position].face() == UnoFace::DrawTwo
                    && deck[position].color() == Some(UnoColor::Red)
                {
                    let replacement = (43..deck.len())
                        .find(|index| {
                            deck[*index].face() != UnoFace::DrawTwo
                                || deck[*index].color() != Some(UnoColor::Red)
                        })
                        .unwrap();
                    deck.swap(position, replacement);
                }
            }
        }

        let mut game = GameState::new_with_deck(
            UnoRuleSet {
                action_stacking: true,
                ..UnoRuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(UnoPlayerId(0), draw_two, None).unwrap();
        game.play_card(UnoPlayerId(1), draw_four, Some(UnoColor::Blue))
            .unwrap();
        let outcome = game.challenge_draw_four(UnoPlayerId(2)).unwrap();
        let expected = if alternative == same_color_number {
            UnoChallengeResult::Failed
        } else {
            UnoChallengeResult::Successful
        };
        assert!(
            matches!(outcome, ActionOutcome::ChallengeResolved { result, .. } if result == expected)
        );
    }
}

#[test]
fn successful_challenge_returns_whole_stack_to_first_offender() {
    let draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let matching = card(UnoColor::Red, UnoFace::Number(7), 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, draw_four), (6, matching), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.play_card(UnoPlayerId(0), draw_four, Some(UnoColor::Blue))
        .unwrap();
    let before = game.player(UnoPlayerId(0)).unwrap().hand().len();
    let outcome = game.challenge_draw_four(UnoPlayerId(1)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::ChallengeResolved {
            result: UnoChallengeResult::Successful,
            penalized: UnoPlayerId(0),
            ref cards,
            next_player: UnoPlayerId(1),
            ..
        } if cards.len() == 4
    ));
    assert_eq!(
        game.player(UnoPlayerId(0)).unwrap().hand().len(),
        before + 4
    );
}

#[test]
fn failed_challenge_adds_two_to_the_whole_stack() {
    let draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, draw_four), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    // Ensure player zero has no red card besides the wild.
    for position in [6, 12, 18, 24, 30, 36] {
        if deck[position].color() == Some(UnoColor::Red) {
            let replacement = (43..deck.len())
                .find(|index| deck[*index].color() != Some(UnoColor::Red))
                .unwrap();
            deck.swap(position, replacement);
        }
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.play_card(UnoPlayerId(0), draw_four, Some(UnoColor::Blue))
        .unwrap();
    let before = game.player(UnoPlayerId(1)).unwrap().hand().len();
    let outcome = game.challenge_draw_four(UnoPlayerId(1)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::ChallengeResolved {
            result: UnoChallengeResult::Failed,
            penalized: UnoPlayerId(1),
            ref cards,
            next_player: UnoPlayerId(2),
            ..
        } if cards.len() == 6
    ));
    assert_eq!(
        game.player(UnoPlayerId(1)).unwrap().hand().len(),
        before + 6
    );
}

#[test]
fn stacked_draw_fours_keep_the_first_player_responsible() {
    let first = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let second = UnoCard::wild(UnoFace::WildDrawFour, 1);
    let matching = card(UnoColor::Red, UnoFace::Number(7), 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, first), (1, second), (6, matching), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(
        UnoRuleSet {
            action_stacking: true,
            ..UnoRuleSet::default()
        },
        6,
        deck,
    )
    .unwrap();
    game.play_card(UnoPlayerId(0), first, Some(UnoColor::Blue))
        .unwrap();
    game.play_card(UnoPlayerId(1), second, Some(UnoColor::Green))
        .unwrap();
    let outcome = game.challenge_draw_four(UnoPlayerId(2)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::ChallengeResolved {
            offender: UnoPlayerId(0),
            result: UnoChallengeResult::Successful,
            penalized: UnoPlayerId(0),
            ref cards,
            ..
        } if cards.len() == 8
    ));
}
