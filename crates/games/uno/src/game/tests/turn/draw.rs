use super::super::prelude::*;

#[test]
fn playable_drawn_card_may_be_played_or_passed_but_no_other_card_can() {
    let drawn = card(UnoColor::Red, UnoFace::Number(9), 0);
    let start = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut deck = deck_with_prefix(&[]);
    for (position, required) in [(42, start), (43, drawn)] {
        let current = deck
            .iter()
            .position(|candidate| *candidate == required)
            .unwrap();
        deck.swap(position, current);
    }
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    let other = game.players[0].hand[0];
    let outcome = game.draw_card(UnoPlayerId(0)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::DrewCards {
            playable: Some(card), ..
        } if card == drawn
    ));
    assert_eq!(
        game.play_card(
            UnoPlayerId(0),
            other,
            other.face().is_wild().then_some(UnoColor::Blue)
        ),
        Err(GameError::MustPlayDrawnCard(drawn))
    );
    game.pass_after_draw(UnoPlayerId(0)).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}
