use super::prelude::*;

fn jump_in_game() -> GameState {
    let mut game = GameState::new_with_deck(
        UnoRuleSet {
            action_stacking: true,
            jump_in: true,
            ..UnoRuleSet::default()
        },
        3,
        build_deck(),
    )
    .unwrap();
    game.direction = UnoDirection::Clockwise;
    game
}

#[test]
fn non_next_player_can_jump_in_and_then_recover_uno_at_one_card() {
    let top = card(UnoColor::Red, UnoFace::Number(7), 0);
    let matching = card(UnoColor::Red, UnoFace::Number(7), 1);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = jump_in_game();
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(1);
    game.jump_in_open = true;
    game.players[2].hand = vec![matching, filler];

    assert_eq!(game.jump_in_card(UnoPlayerId(1)), None);
    assert_eq!(game.jump_in_card(UnoPlayerId(2)), Some(matching));
    assert!(matches!(
        game.jump_in(UnoPlayerId(2), matching),
        Ok(ActionOutcome::Played {
            player: UnoPlayerId(2),
            next_player: UnoPlayerId(0),
            ..
        })
    ));
    assert_eq!(game.player(UnoPlayerId(2)).unwrap().hand(), &[filler]);
    assert_eq!(
        game.uno_exposed_players().collect::<Vec<_>>(),
        vec![UnoPlayerId(2)]
    );
    assert!(matches!(
        game.call_uno(UnoPlayerId(2)),
        Ok(ActionOutcome::UnoCalled {
            player: UnoPlayerId(2)
        })
    ));
}

#[test]
fn jump_in_without_action_stacking_only_offers_number_cards() {
    let number_top = card(UnoColor::Red, UnoFace::Number(7), 0);
    let number_match = card(UnoColor::Red, UnoFace::Number(7), 1);
    let reverse_top = card(UnoColor::Red, UnoFace::Reverse, 0);
    let reverse_match = card(UnoColor::Red, UnoFace::Reverse, 1);
    let mut game = GameState::new_with_deck(
        UnoRuleSet {
            jump_in: true,
            ..UnoRuleSet::default()
        },
        3,
        build_deck(),
    )
    .unwrap();
    game.current_player = UnoPlayerId(1);
    game.jump_in_open = true;
    game.discard_pile = vec![number_top];
    game.current_color = Some(UnoColor::Red);
    game.players[2].hand = vec![number_match, reverse_match];

    assert_eq!(game.jump_in_card(UnoPlayerId(2)), Some(number_match));

    game.discard_pile = vec![reverse_top];
    assert_eq!(game.jump_in_card(UnoPlayerId(2)), None);
}

#[test]
fn current_players_first_successful_action_closes_the_jump_window() {
    let top = card(UnoColor::Red, UnoFace::Number(7), 0);
    let matching = card(UnoColor::Red, UnoFace::Number(7), 1);
    let mut game = jump_in_game();
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(1);
    game.jump_in_open = true;
    game.players[2].hand = vec![matching, card(UnoColor::Blue, UnoFace::Number(3), 0)];

    assert_eq!(game.jump_in_card(UnoPlayerId(2)), Some(matching));
    game.draw_card(UnoPlayerId(1)).unwrap();
    assert_eq!(game.jump_in_card(UnoPlayerId(2)), None);
    assert_eq!(
        game.jump_in(UnoPlayerId(2), matching),
        Err(GameError::CannotJumpIn)
    );
}

#[test]
fn jumped_draw_two_and_skip_extend_the_existing_stack() {
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);

    let draw_top = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let draw_match = card(UnoColor::Red, UnoFace::DrawTwo, 1);
    let mut draw_game = jump_in_game();
    draw_game.discard_pile = vec![draw_top];
    draw_game.current_color = Some(UnoColor::Red);
    draw_game.current_player = UnoPlayerId(1);
    draw_game.pending_draw = 2;
    draw_game.pending_kind = Some(UnoPendingDrawKind::DrawTwo);
    draw_game.jump_in_open = true;
    draw_game.players[2].hand = vec![draw_match, filler];
    draw_game.jump_in(UnoPlayerId(2), draw_match).unwrap();
    assert_eq!(draw_game.turn().unwrap().pending_draw, 4);
    assert_eq!(draw_game.turn().unwrap().current_player, UnoPlayerId(0));

    let skip_top = card(UnoColor::Yellow, UnoFace::Skip, 0);
    let skip_match = card(UnoColor::Yellow, UnoFace::Skip, 1);
    let mut skip_game = jump_in_game();
    skip_game.discard_pile = vec![skip_top];
    skip_game.current_color = Some(UnoColor::Yellow);
    skip_game.current_player = UnoPlayerId(1);
    skip_game.pending_skip = 1;
    skip_game.jump_in_open = true;
    skip_game.players[2].hand = vec![skip_match, filler];
    skip_game.jump_in(UnoPlayerId(2), skip_match).unwrap();
    assert_eq!(skip_game.turn().unwrap().pending_skip, 2);
    assert_eq!(skip_game.turn().unwrap().current_player, UnoPlayerId(0));
}

#[test]
fn identical_pair_is_atomic_and_two_reverses_restore_direction() {
    let first = card(UnoColor::Red, UnoFace::Reverse, 0);
    let second = card(UnoColor::Red, UnoFace::Reverse, 1);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = jump_in_game();
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.players[0].hand = vec![first, second, filler];

    assert!(matches!(
        game.play_cards(UnoPlayerId(0), &[first, second], None),
        Ok(ActionOutcome::Played {
            player: UnoPlayerId(0),
            next_player: UnoPlayerId(1),
            ..
        })
    ));
    assert_eq!(game.direction(), UnoDirection::Clockwise);
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand(), &[filler]);
    assert_eq!(
        &game.discard_pile()[game.discard_pile().len() - 2..],
        &[first, second]
    );
}

#[test]
fn identical_last_pair_finishes_together_but_declared_uno_forces_one_card() {
    let first = card(UnoColor::Green, UnoFace::Number(8), 0);
    let second = card(UnoColor::Green, UnoFace::Number(8), 1);
    let mut game = jump_in_game();
    game.discard_pile = vec![card(UnoColor::Green, UnoFace::Number(4), 0)];
    game.current_color = Some(UnoColor::Green);
    game.current_player = UnoPlayerId(0);
    game.players[0].hand = vec![first, second];
    assert!(matches!(
        game.play_cards(UnoPlayerId(0), &[first, second], None),
        Ok(ActionOutcome::Played {
            player: UnoPlayerId(0),
            ..
        })
    ));
    assert!(matches!(game.phase(), Phase::Finished(_)));

    let mut declared = jump_in_game();
    declared.discard_pile = vec![card(UnoColor::Green, UnoFace::Number(4), 0)];
    declared.current_color = Some(UnoColor::Green);
    declared.current_player = UnoPlayerId(0);
    declared.players[0].hand = vec![first, second];
    declared.uno_declared[0] = true;
    assert_eq!(
        declared.play_cards(UnoPlayerId(0), &[first, second], None),
        Err(GameError::CannotPlayTogether)
    );
    assert_eq!(
        declared.player(UnoPlayerId(0)).unwrap().hand(),
        &[first, second]
    );
}
