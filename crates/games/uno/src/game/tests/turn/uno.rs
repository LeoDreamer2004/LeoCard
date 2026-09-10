use super::super::prelude::*;

#[test]
fn uno_may_be_called_before_or_immediately_after_the_penultimate_card() {
    let mut game = GameState::new_with_deck(UnoRuleSet::default(), 6, build_deck()).unwrap();
    game.players[0].hand.truncate(2);
    let playable = game.players[0]
        .hand
        .iter()
        .copied()
        .find(|card| game.matches_top(*card))
        .unwrap_or_else(|| {
            let card = *game.discard_pile.last().unwrap();
            game.players[0].hand[0] = card;
            card
        });
    assert!(game.can_call_uno(UnoPlayerId(0)));
    game.call_uno(UnoPlayerId(0)).unwrap();
    assert!(!game.can_call_uno(UnoPlayerId(0)));
    assert_eq!(
        game.uno_declared_players().collect::<Vec<_>>(),
        vec![UnoPlayerId(0)]
    );
    assert_eq!(
        game.draw_card(UnoPlayerId(0)),
        Err(GameError::MustPlayAfterUno)
    );
    game.play_card(UnoPlayerId(0), playable, None).unwrap();
    assert!(game.uno_exposed_players().next().is_none());

    game.current_player = UnoPlayerId(1);
    game.players[1].hand.truncate(2);
    let playable = game.players[1]
        .hand
        .iter()
        .copied()
        .find(|card| game.matches_top(*card))
        .unwrap_or_else(|| {
            let card = *game.discard_pile.last().unwrap();
            game.players[1].hand[0] = card;
            card
        });
    game.play_card(UnoPlayerId(1), playable, None).unwrap();
    assert_eq!(
        game.uno_exposed_players().collect::<Vec<_>>(),
        vec![UnoPlayerId(1)]
    );
    assert!(matches!(
        game.call_uno(UnoPlayerId(1)),
        Ok(ActionOutcome::UnoCalled {
            player: UnoPlayerId(1)
        })
    ));
    assert!(game.uno_exposed_players().next().is_none());
    assert!(game.uno_declared_players().next().is_none());

    // 补喊后不能被重复检举，也不会留下“必须立刻出牌”的预喊状态。
    let before = game.player(UnoPlayerId(1)).unwrap().hand().len();
    assert_eq!(
        game.report_uno(UnoPlayerId(2), UnoPlayerId(1)),
        Err(GameError::PlayerNotReportable(UnoPlayerId(1)))
    );
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand().len(), before);
}

#[test]
fn eliminated_players_cannot_call_or_report_uno() {
    let mut game = no_mercy_game(3);
    game.players[2].eliminated = true;
    game.uno_exposed[1] = true;

    assert_eq!(
        game.call_uno(UnoPlayerId(2)),
        Err(GameError::PlayerEliminated(UnoPlayerId(2)))
    );
    assert_eq!(
        game.report_uno(UnoPlayerId(2), UnoPlayerId(1)),
        Err(GameError::PlayerEliminated(UnoPlayerId(2)))
    );
}

#[test]
fn uno_may_be_called_after_drawing_a_playable_card_from_one_to_two() {
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
    game.players[0].hand.truncate(1);

    assert!(matches!(
        game.draw_card(UnoPlayerId(0)),
        Ok(ActionOutcome::DrewCards {
            cards,
            playable: Some(card),
            ..
        }) if cards == vec![drawn] && card == drawn
    ));
    assert!(matches!(
        game.call_uno(UnoPlayerId(0)),
        Ok(ActionOutcome::UnoCalled {
            player: UnoPlayerId(0)
        })
    ));
    game.play_card(UnoPlayerId(0), drawn, None).unwrap();
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand().len(), 1);
    assert!(game.uno_exposed_players().next().is_none());
    assert!(game.uno_declared_players().next().is_none());
}
