use super::super::prelude::*;

#[test]
fn last_draw_four_finishes_after_the_penalty_is_resolved() {
    let wild_draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = build_deck();
    for (position, required) in [(0, wild_draw_four), (42, start)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.players[0].hand = vec![wild_draw_four];
    let target_hand = game.players[1].hand.len();
    let outcome = game
        .play_card(UnoPlayerId(0), wild_draw_four, Some(UnoColor::Blue))
        .unwrap();
    assert!(matches!(outcome, ActionOutcome::Played { .. }));
    let turn = game
        .turn()
        .expect("the final draw penalty still needs resolving");
    assert_eq!(turn.current_color, Some(UnoColor::Blue));
    assert_eq!(turn.current_player, UnoPlayerId(1));
    assert_eq!(turn.pending_draw, 4);
    assert_eq!(turn.challenge_offender, Some(UnoPlayerId(0)));

    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 4
    ));
    assert_eq!(game.players[1].hand.len(), target_hand + 4);
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
    assert_eq!(
        game.challenge_draw_four(UnoPlayerId(1)),
        Err(GameError::GameAlreadyFinished)
    );
}

#[test]
fn last_skip_finishes_after_the_target_loses_their_turn() {
    let skip = card(UnoColor::Red, UnoFace::Skip, 0);
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, build_deck()).unwrap();
    game.players[0].hand = vec![skip];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.play_card(UnoPlayerId(0), skip, None),
        Ok(ActionOutcome::Played { .. })
    ));
    assert_eq!(game.skipped_turns(UnoPlayerId(1)), Some(1));
    assert!(matches!(game.phase(), Phase::Playing));

    assert!(matches!(
        game.resolve_skip(UnoPlayerId(1)),
        Ok(ActionOutcome::SkipResolved {
            player: UnoPlayerId(1),
            remaining: 0,
            ..
        })
    ));
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}
