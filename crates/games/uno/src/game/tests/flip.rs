use super::prelude::*;

#[test]
fn deals_seven_cards_and_applies_number_start() {
    let game = GameState::new_with_deck(UnoRuleSet::default(), 6, build_deck()).unwrap();
    assert!(game.players().iter().all(|player| player.hand().len() == 7));
    assert_eq!(game.players().len(), 6);
    assert_eq!(game.draw_pile_len(), 65);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(0));
}

#[test]
fn flip_starting_card_turns_every_physical_card_to_the_dark_side() {
    let mut deck = build_flip_deck();
    let start = usize::from(UnoRuleSet::HAND_SIZE) * 2;
    let flip = deck
        .iter()
        .position(|card| card.face() == UnoFace::Flip)
        .unwrap();
    deck.swap(start, flip);
    let game = GameState::new_with_deck(flip_rules(), 2, deck).unwrap();
    assert_eq!(game.flip_side(), Some(UnoFlipSide::Dark));
    assert!(
        game.players()
            .iter()
            .flat_map(PlayerState::hand)
            .all(|card| {
                card.color().is_none_or(UnoColor::is_dark)
                    && card
                        .opposite()
                        .is_some_and(|side| side.color().is_none_or(UnoColor::is_light))
            })
    );
    assert!(game.top_card().color().is_none_or(UnoColor::is_dark));
}

#[test]
fn dark_side_keeps_uno_calls_and_reports_enabled() {
    fn dark_card(color: UnoColor, face: UnoFace, copy: u8) -> UnoCard {
        UnoCard::paired(
            CardSide::colored(UnoColor::Red, UnoFace::Number(1)),
            CardSide::colored(color, face),
            copy,
        )
        .flipped()
    }

    let playable = dark_card(UnoColor::Pink, UnoFace::Number(3), 0);
    let remaining = dark_card(UnoColor::Teal, UnoFace::Number(4), 0);
    let top = dark_card(UnoColor::Pink, UnoFace::Number(8), 0);

    let mut declared = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
    declared.flip_side = Some(UnoFlipSide::Dark);
    declared.players[0].hand = vec![playable, remaining];
    declared.discard_pile = vec![top];
    declared.current_color = Some(UnoColor::Pink);
    declared.current_player = UnoPlayerId(0);
    assert!(matches!(
        declared.call_uno(UnoPlayerId(0)),
        Ok(ActionOutcome::UnoCalled {
            player: UnoPlayerId(0)
        })
    ));
    declared.play_card(UnoPlayerId(0), playable, None).unwrap();
    assert!(declared.uno_exposed_players().next().is_none());

    let mut exposed = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
    exposed.flip_side = Some(UnoFlipSide::Dark);
    exposed.players[0].hand = vec![playable, remaining];
    exposed.discard_pile = vec![top];
    exposed.current_color = Some(UnoColor::Pink);
    exposed.current_player = UnoPlayerId(0);
    exposed.play_card(UnoPlayerId(0), playable, None).unwrap();
    assert_eq!(
        exposed.uno_exposed_players().collect::<Vec<_>>(),
        vec![UnoPlayerId(0)]
    );
    assert!(matches!(
        exposed.report_uno(UnoPlayerId(1), UnoPlayerId(0)),
        Ok(ActionOutcome::UnoReported {
            reporter: UnoPlayerId(1),
            target: UnoPlayerId(0),
            ref cards,
        }) if cards.len() == 2
    ));
}

#[test]
fn flipping_reverses_draw_and_discard_order_while_swapping_every_face() {
    let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
    let extra = [
        game.draw_pile.pop_front().unwrap(),
        game.draw_pile.pop_front().unwrap(),
    ];
    game.discard_pile.extend(extra);
    let old_draw = game.draw_pile.iter().copied().collect::<Vec<_>>();
    let old_discard = game.discard_pile.clone();
    game.flip_everything();
    assert_eq!(
        game.discard_pile,
        old_discard
            .into_iter()
            .rev()
            .map(UnoCard::flipped)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        game.draw_pile.back().copied(),
        old_draw.first().copied().map(UnoCard::flipped)
    );
}

#[test]
fn flip_penalty_stacks_use_four_separate_chains() {
    let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
    game.rules.flip.action_stacking = true;
    let draw_one = flip_card(UnoColor::Red, UnoFace::DrawOne);
    let wild_two = UnoCard::paired(
        CardSide::wild(UnoFace::WildDrawTwo),
        CardSide::wild(UnoFace::WildDrawColor),
        0,
    );
    let draw_five = flip_card(UnoColor::Pink, UnoFace::DrawFive);
    let wild_color = UnoCard::paired(
        CardSide::wild(UnoFace::WildDrawColor),
        CardSide::wild(UnoFace::WildDrawTwo),
        0,
    );
    game.pending_kind = Some(UnoPendingDrawKind::FlipDrawOne);
    assert!(game.stack_allowed(draw_one));
    assert!(game.stack_allowed(wild_two));
    assert!(!game.stack_allowed(draw_five));
    game.pending_kind = Some(UnoPendingDrawKind::FlipWildDrawTwo);
    assert!(game.stack_allowed(wild_two));
    assert!(!game.stack_allowed(draw_one));
    game.pending_kind = Some(UnoPendingDrawKind::FlipDrawFive);
    assert!(game.stack_allowed(draw_five));
    assert!(!game.stack_allowed(wild_color));
    game.pending_kind = Some(UnoPendingDrawKind::FlipWildDrawColor);
    assert!(game.stack_allowed(wild_color));
    assert!(!game.stack_allowed(draw_five));
}

#[test]
fn stacked_wild_draw_colors_resolve_each_selected_color_in_order() {
    let mut game = GameState::new_with_deck(flip_rules(), 2, build_flip_deck()).unwrap();
    game.pending_kind = Some(UnoPendingDrawKind::FlipWildDrawColor);
    game.pending_draw = 2;
    game.pending_draw_colors = vec![UnoColor::Red, UnoColor::Blue];
    game.draw_pile = VecDeque::from(vec![
        flip_card(UnoColor::Green, UnoFace::Number(1)),
        flip_card(UnoColor::Red, UnoFace::Number(2)),
        flip_card(UnoColor::Yellow, UnoFace::Number(3)),
        flip_card(UnoColor::Blue, UnoFace::Number(4)),
        flip_card(UnoColor::Red, UnoFace::Number(5)),
    ]);
    let cards = game.draw_pending_penalty(UnoPlayerId(0), 0).unwrap();
    assert_eq!(cards.len(), 4);
    assert_eq!(cards[1].color(), Some(UnoColor::Red));
    assert_eq!(cards[3].color(), Some(UnoColor::Blue));
}

#[test]
fn identical_flip_pair_cancels_without_turning_the_table_over() {
    let mut rules = flip_rules();
    rules.flip.action_stacking = true;
    rules.flip.jump_in = true;
    let mut game = GameState::new_with_deck(rules, 2, build_flip_deck()).unwrap();
    let pair = build_flip_deck()
        .into_iter()
        .filter(|card| card.color() == Some(UnoColor::Red) && card.face() == UnoFace::Flip)
        .collect::<Vec<_>>();
    game.players[0].hand = pair.clone();
    game.current_player = UnoPlayerId(0);
    game.current_color = Some(UnoColor::Red);
    game.discard_pile = vec![flip_card(UnoColor::Red, UnoFace::Number(4))];
    game.play_cards(UnoPlayerId(0), &pair, None).unwrap();
    assert_eq!(game.flip_side(), Some(UnoFlipSide::Light));
    assert!(matches!(game.phase(), Phase::Finished(_)));
    assert!(
        game.discard_pile()
            .iter()
            .rev()
            .take(2)
            .all(|card| card.face() == UnoFace::Flip)
    );
}

#[test]
fn stacked_dark_skip_everyone_is_consumed_by_every_other_player() {
    let mut rules = flip_rules();
    rules.flip.action_stacking = true;
    let mut game = GameState::new_with_deck(rules, 3, build_flip_deck()).unwrap();
    game.flip_everything();
    let skip = build_flip_deck()
        .into_iter()
        .map(UnoCard::flipped)
        .find(|card| card.color() == Some(UnoColor::Pink) && card.face() == UnoFace::SkipEveryone)
        .unwrap();
    game.players[0].hand = vec![skip];
    game.current_player = UnoPlayerId(0);
    game.direction = UnoDirection::Clockwise;
    game.current_color = Some(UnoColor::Pink);
    game.discard_pile = vec![UnoCard::paired(
        CardSide::colored(UnoColor::Pink, UnoFace::Number(4)),
        CardSide::colored(UnoColor::Red, UnoFace::Number(4)),
        120,
    )];
    game.play_card(UnoPlayerId(0), skip, None).unwrap();
    assert!(matches!(game.phase(), Phase::Playing));
    game.resolve_skip(UnoPlayerId(1)).unwrap();
    assert!(matches!(game.phase(), Phase::Playing));
    game.resolve_skip(UnoPlayerId(2)).unwrap();
    assert!(matches!(game.phase(), Phase::Finished(_)));
}

#[test]
fn two_to_six_players_are_supported_but_seven_are_rejected() {
    for player_count in UnoRuleSet::MIN_PLAYERS..=UnoRuleSet::MAX_PLAYERS {
        let game =
            GameState::new_with_deck(UnoRuleSet::default(), player_count, build_deck()).unwrap();
        assert_eq!(game.players().len(), usize::from(player_count));
        assert!(game.players().iter().all(|player| player.hand().len() == 7));
    }
    assert!(matches!(
        GameState::new_with_deck(UnoRuleSet::default(), 7, build_deck()),
        Err(GameError::InvalidRules(_))
    ));
}
