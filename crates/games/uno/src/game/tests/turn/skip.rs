use super::super::prelude::*;

#[test]
fn skip_stacking_transfers_multiple_blocked_turns() {
    let first = card(UnoColor::Red, UnoFace::Skip, 0);
    let second = card(UnoColor::Yellow, UnoFace::Skip, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, first), (1, second), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(
        UnoRuleSet {
            action_stacking: true,
            skip_draw_penalty: true,
            ..UnoRuleSet::default()
        },
        6,
        deck,
    )
    .unwrap();
    game.play_card(UnoPlayerId(0), first, None).unwrap();
    assert_eq!(game.turn().unwrap().pending_skip, 1);
    game.play_card(UnoPlayerId(1), second, None).unwrap();
    assert_eq!(game.turn().unwrap().pending_skip, 2);
    let before = game.player(UnoPlayerId(2)).unwrap().hand().len();
    let outcome = game.resolve_skip(UnoPlayerId(2)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::SkipResolved {
            player: UnoPlayerId(2),
            remaining: 1,
            ref cards,
            next_player: UnoPlayerId(3),
        } if cards.len() == 1
    ));
    assert_eq!(game.skipped_turns(UnoPlayerId(2)), Some(1));
    assert_eq!(
        game.player(UnoPlayerId(2)).unwrap().hand().len(),
        before + 1
    );

    game.current_player = UnoPlayerId(2);
    game.resolve_skip(UnoPlayerId(2)).unwrap();
    assert_eq!(game.skipped_turns(UnoPlayerId(2)), Some(0));
    assert_eq!(
        game.player(UnoPlayerId(2)).unwrap().hand().len(),
        before + 2
    );
}

#[test]
fn ordinary_skip_blocks_exactly_one_turn() {
    let skip = card(UnoColor::Red, UnoFace::Skip, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, skip), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.play_card(UnoPlayerId(0), skip, None).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
    assert_eq!(game.skipped_turns(UnoPlayerId(1)), Some(1));
    assert_eq!(
        game.draw_card(UnoPlayerId(1)),
        Err(GameError::MustResolveSkip)
    );
    game.resolve_skip(UnoPlayerId(1)).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(2));
    assert_eq!(game.skipped_turns(UnoPlayerId(1)), Some(0));
}

#[test]
fn draw_two_is_paid_by_the_skipped_direct_next_player_before_skip_resolves() {
    let draw_two = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, draw_two), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.skip_turns[1] = 1;
    let before = game.player(UnoPlayerId(1)).unwrap().hand().len();

    game.play_card(UnoPlayerId(0), draw_two, None).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
    assert_eq!(
        game.resolve_skip(UnoPlayerId(1)),
        Err(GameError::MustResolveDrawPenalty)
    );

    let outcome = game.accept_draw_penalty(UnoPlayerId(1)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::PenaltyDrawn {
            player: UnoPlayerId(1),
            ref cards,
            next_player: UnoPlayerId(1),
        } if cards.len() == 2
    ));
    assert_eq!(
        game.player(UnoPlayerId(1)).unwrap().hand().len(),
        before + 2
    );
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
    assert_eq!(game.turn().unwrap().pending_draw, 0);
    assert_eq!(game.skipped_turns(UnoPlayerId(1)), Some(1));

    game.resolve_skip(UnoPlayerId(1)).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(2));
    assert_eq!(game.skipped_turns(UnoPlayerId(1)), Some(0));
}
