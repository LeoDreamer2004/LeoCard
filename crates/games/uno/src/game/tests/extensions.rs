use super::*;
fn swap_pack_game() -> GameState {
    let rules = UnoRuleSet {
        swap_pack: true,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

fn reverse_pack_game() -> GameState {
    let rules = UnoRuleSet {
        reverse_pack: true,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

fn stack_pack_game() -> GameState {
    let rules = UnoRuleSet {
        stack_pack: true,
        ..UnoRuleSet::default()
    };
    GameState::new_with_deck(rules, 6, build_deck_for_rules(rules)).unwrap()
}

#[test]
fn colored_stack_cards_require_the_current_color_and_accumulate() {
    let stack_one = card(UnoColor::Red, UnoFace::StackOne, 0);
    let matching_stack_two = card(UnoColor::Red, UnoFace::StackTwo, 0);
    let wrong_stack_two = card(UnoColor::Blue, UnoFace::StackTwo, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![stack_one, filler];
    game.players[1].hand = vec![matching_stack_two, wrong_stack_two, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), stack_one, None).unwrap();
    assert!(!game.can_play(UnoPlayerId(1), wrong_stack_two));
    assert!(game.can_play(UnoPlayerId(1), matching_stack_two));
    game.play_card(UnoPlayerId(1), matching_stack_two, None)
        .unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.pending_draw, 3);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::Stack));
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(1)));
    assert_eq!(turn.current_player, UnoPlayerId(2));
}

#[test]
fn same_stack_face_does_not_bypass_its_color_requirement() {
    let blue_stack = card(UnoColor::Blue, UnoFace::StackOne, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![blue_stack, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::StackOne, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(!game.can_play(UnoPlayerId(0), blue_stack));
    assert_eq!(
        game.play_card(UnoPlayerId(0), blue_stack, None),
        Err(GameError::CardDoesNotMatch)
    );
}

#[test]
fn wild_stack_number_reveals_to_a_number_below_the_discard_top() {
    let stack_number = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let revealed_action = card(UnoColor::Blue, UnoFace::Skip, 0);
    let revealed_number = card(UnoColor::Yellow, UnoFace::Number(6), 0);
    let old_top = card(UnoColor::Red, UnoFace::Number(7), 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_number, filler];
    game.discard_pile = vec![old_top];
    game.draw_pile = VecDeque::from([revealed_action, revealed_number, filler]);
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    let outcome = game
        .play_card(UnoPlayerId(0), stack_number, Some(UnoColor::Blue))
        .unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::Played {
            effect: Some(PlayedEffect::StackNumberRevealed { ref cards, value: 6 }),
            ..
        } if cards == &[revealed_action, revealed_number]
    ));
    assert_eq!(
        game.discard_pile(),
        &[revealed_action, revealed_number, old_top, stack_number]
    );
    let turn = game.turn().unwrap();
    assert_eq!(turn.pending_draw, 6);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::Stack));
    assert_eq!(turn.current_color, Some(UnoColor::Blue));
}

#[test]
fn stack_number_zero_is_still_an_active_penalty_turn() {
    let stack_number = UnoCard::wild(UnoFace::WildStackNumber, 0);
    let zero = card(UnoColor::Yellow, UnoFace::Number(0), 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_number, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.draw_pile = VecDeque::from([zero, filler]);
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), stack_number, Some(UnoColor::Yellow))
        .unwrap();
    assert_eq!(game.turn().unwrap().pending_draw, 0);
    assert_eq!(
        game.turn().unwrap().pending_kind,
        Some(UnoPendingDrawKind::Stack)
    );
    assert_eq!(
        game.draw_card(UnoPlayerId(1)),
        Err(GameError::MustResolveDrawPenalty)
    );
    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn {
            ref cards,
            next_player: UnoPlayerId(2),
            ..
        }) if cards.is_empty()
    ));
}

#[test]
fn stack_cards_preserve_the_first_draw_four_challenge_offender() {
    let draw_four = UnoCard::wild(UnoFace::WildDrawFour, 0);
    let matching = card(UnoColor::Blue, UnoFace::Number(7), 0);
    let stack_one = card(UnoColor::Red, UnoFace::StackOne, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = stack_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![draw_four, matching];
    game.players[1].hand = vec![stack_one, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), draw_four, Some(UnoColor::Red))
        .unwrap();
    game.play_card(UnoPlayerId(1), stack_one, None).unwrap();
    let outcome = game.challenge_draw_four(UnoPlayerId(2)).unwrap();
    assert!(matches!(
        outcome,
        ActionOutcome::ChallengeResolved {
            offender: UnoPlayerId(0),
            result: UnoChallengeResult::Successful,
            penalized: UnoPlayerId(0),
            ref cards,
            ..
        } if cards.len() == 5
    ));
}

#[test]
fn last_stack_card_finishes_after_applying_its_penalty() {
    let stack_two = card(UnoColor::Red, UnoFace::StackTwo, 0);
    let mut game = stack_pack_game();
    game.players[0].hand = vec![stack_two];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.play_card(UnoPlayerId(0), stack_two, None),
        Ok(ActionOutcome::Played { .. })
    ));
    assert_eq!(game.turn().unwrap().pending_draw, 2);
    let before = game.players[1].hand.len();
    assert!(matches!(
        game.accept_draw_penalty(UnoPlayerId(1)),
        Ok(ActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 2
    ));
    assert_eq!(game.players[1].hand.len(), before + 2);
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}

#[test]
fn reverse_draw_two_matches_reverse_and_redirects_the_penalty() {
    let reverse_draw = card(UnoColor::Red, UnoFace::ReverseDrawTwo, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![reverse_draw, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Reverse, 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    assert!(game.can_play(UnoPlayerId(0), reverse_draw));
    game.play_card(UnoPlayerId(0), reverse_draw, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_draw, 2);
    assert_eq!(turn.pending_kind, Some(UnoPendingDrawKind::DrawTwo));
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(0)));
    assert_eq!(turn.current_player, UnoPlayerId(5));
}

#[test]
fn reverse_skip_changes_direction_before_choosing_the_skipped_player() {
    let reverse_skip = card(UnoColor::Red, UnoFace::ReverseSkip, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![reverse_skip, filler];
    game.discard_pile = vec![card(UnoColor::Blue, UnoFace::Skip, 0)];
    game.current_color = Some(UnoColor::Blue);
    game.current_player = UnoPlayerId(0);

    assert!(game.can_play(UnoPlayerId(0), reverse_skip));
    game.play_card(UnoPlayerId(0), reverse_skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_player, UnoPlayerId(5));
    assert_eq!(game.skipped_turns(UnoPlayerId(5)), Some(1));
}

#[test]
fn power_reverse_changes_color_and_gives_the_actor_another_turn() {
    let power = UnoCard::wild(UnoFace::WildPowerReverse, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.players[0].hand = vec![power, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), power, Some(UnoColor::Green))
        .unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_color, Some(UnoColor::Green));
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn no_u_reflects_the_whole_penalty_to_the_latest_stacker() {
    let draw_a = card(UnoColor::Red, UnoFace::DrawTwo, 0);
    let draw_b = card(UnoColor::Blue, UnoFace::DrawTwo, 0);
    let no_u = UnoCard::wild(UnoFace::WildNoU, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![draw_a, filler];
    game.players[1].hand = vec![draw_b, filler];
    game.players[2].hand = vec![no_u, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), draw_a, None).unwrap();
    game.play_card(UnoPlayerId(1), draw_b, None).unwrap();
    let before = game.player(UnoPlayerId(1)).unwrap().hand().len();
    let outcome = game
        .play_card(UnoPlayerId(2), no_u, Some(UnoColor::Yellow))
        .unwrap();

    assert!(matches!(
        outcome,
        ActionOutcome::Played {
            effect: Some(PlayedEffect::DrawReflected {
                player: UnoPlayerId(1),
                ref cards,
            }),
            ..
        } if cards.len() == 4
    ));
    assert_eq!(
        game.player(UnoPlayerId(1)).unwrap().hand().len(),
        before + 4
    );
    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.current_color, Some(UnoColor::Yellow));
    assert_eq!(turn.pending_draw, 0);
    assert_eq!(turn.pending_draw_source, None);
    assert_eq!(turn.challenge_offender, None);
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn reverse_skip_can_redirect_an_accumulated_skip() {
    let skip = card(UnoColor::Red, UnoFace::Skip, 0);
    let reverse_skip = card(UnoColor::Blue, UnoFace::ReverseSkip, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![skip, filler];
    game.players[1].hand = vec![reverse_skip, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), skip, None).unwrap();
    assert!(game.can_play(UnoPlayerId(1), reverse_skip));
    game.play_card(UnoPlayerId(1), reverse_skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_skip, 2);
    assert_eq!(turn.current_player, UnoPlayerId(0));
}

#[test]
fn ordinary_skip_can_stack_on_reverse_skip() {
    let reverse_skip = card(UnoColor::Red, UnoFace::ReverseSkip, 0);
    let skip = card(UnoColor::Blue, UnoFace::Skip, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![reverse_skip, filler];
    game.players[5].hand = vec![skip, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), reverse_skip, None).unwrap();
    assert!(game.can_play(UnoPlayerId(5), skip));
    game.play_card(UnoPlayerId(5), skip, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_skip, 2);
    assert_eq!(turn.current_player, UnoPlayerId(4));
}

#[test]
fn ordinary_draw_two_can_stack_on_reverse_draw_two() {
    let reverse_draw = card(UnoColor::Red, UnoFace::ReverseDrawTwo, 0);
    let draw_two = card(UnoColor::Blue, UnoFace::DrawTwo, 0);
    let filler = card(UnoColor::Green, UnoFace::Number(3), 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![reverse_draw, filler];
    game.players[5].hand = vec![draw_two, filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(7), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), reverse_draw, None).unwrap();
    assert!(game.can_play(UnoPlayerId(5), draw_two));
    game.play_card(UnoPlayerId(5), draw_two, None).unwrap();

    let turn = game.turn().unwrap();
    assert_eq!(turn.direction, UnoDirection::CounterClockwise);
    assert_eq!(turn.pending_draw, 4);
    assert_eq!(turn.pending_draw_source, Some(UnoPlayerId(5)));
    assert_eq!(turn.current_player, UnoPlayerId(4));
}

#[test]
fn last_no_u_reflects_the_pending_penalty_before_winning() {
    let no_u = UnoCard::wild(UnoFace::WildNoU, 0);
    let mut game = reverse_pack_game();
    game.rules.action_stacking = true;
    game.players[0].hand = vec![no_u];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 2;
    game.pending_kind = Some(UnoPendingDrawKind::DrawTwo);
    game.pending_draw_source = Some(UnoPlayerId(5));
    let source_hand_len = game.player(UnoPlayerId(5)).unwrap().hand().len();

    assert!(matches!(
        game.play_card(UnoPlayerId(0), no_u, Some(UnoColor::Blue)),
        Ok(ActionOutcome::Played {
            effect: Some(PlayedEffect::DrawReflected {
                player: UnoPlayerId(5),
                ref cards,
            }),
            ..
        }) if cards.len() == 2
    ));
    assert_eq!(
        game.player(UnoPlayerId(5)).unwrap().hand().len(),
        source_hand_len + 2
    );
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}

#[test]
fn refresh_hand_places_old_cards_under_the_discard_and_draws_the_same_count() {
    let refresh = card(UnoColor::Red, UnoFace::RefreshHand, 0);
    let first = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let second = card(UnoColor::Green, UnoFace::Number(2), 0);
    let top = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![refresh, first, second];
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.rules.action_stacking = true;
    game.rules.jump_in = true;

    let outcome = game.play_card(UnoPlayerId(0), refresh, None).unwrap();

    assert!(matches!(
        outcome,
        ActionOutcome::Played {
            effect: Some(PlayedEffect::HandRefreshed { count: 2 }),
            ..
        }
    ));
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand().len(), 2);
    assert_eq!(game.discard_pile(), &[first, second, top, refresh]);
    assert_eq!(game.top_card(), refresh);
    assert!(!game.jump_in_open);
}

#[test]
fn swap_one_allows_returning_the_taken_card_without_creating_uno_state() {
    let swap = card(UnoColor::Red, UnoFace::SwapOne, 0);
    let filler = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let taken = card(UnoColor::Yellow, UnoFace::Number(4), 0);
    let target_filler = card(UnoColor::Green, UnoFace::Number(6), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![swap, filler];
    game.players[1].hand = vec![taken, target_filler];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), swap, None).unwrap();
    game.choose_swap_one_target(UnoPlayerId(0), UnoPlayerId(1), 0)
        .unwrap();
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand().len(), 1);
    assert!(game.uno_exposed_players().next().is_none());
    assert!(game.player(UnoPlayerId(0)).unwrap().hand().contains(&taken));

    game.give_swap_one_card(UnoPlayerId(0), taken).unwrap();
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand().len(), 2);
    assert!(game.player(UnoPlayerId(1)).unwrap().hand().contains(&taken));
    assert!(
        !game
            .uno_exposed_players()
            .any(|player| player == UnoPlayerId(1))
    );
}

#[test]
fn force_trade_may_include_actor_and_defers_color_until_after_confirmation() {
    let trade = UnoCard::wild(UnoFace::WildForceTrade, 0);
    let actor_card = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let target_card = card(UnoColor::Yellow, UnoFace::Number(2), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![trade, actor_card];
    game.players[1].hand = vec![target_card];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.uno_exposed[1] = true;

    game.play_card(UnoPlayerId(0), trade, None).unwrap();
    assert_eq!(game.current_color(), None);
    assert_eq!(
        game.choose_initial_color(UnoPlayerId(0), UnoColor::Green),
        Err(GameError::MustResolveSwapEffect)
    );
    game.force_trade_hands(UnoPlayerId(0), UnoPlayerId(0), UnoPlayerId(1))
        .unwrap();
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand(), &[target_card]);
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand(), &[actor_card]);
    assert!(game.uno_exposed_players().next().is_none());

    game.choose_initial_color(UnoPlayerId(0), UnoColor::Green)
        .unwrap();
    assert_eq!(game.current_color(), Some(UnoColor::Green));
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn pass_hands_follows_direction_and_clears_every_uno_state() {
    let pass = UnoCard::wild(UnoFace::WildPassHands, 0);
    let mut game = swap_pack_game();
    let markers = [
        card(UnoColor::Red, UnoFace::Number(1), 0),
        card(UnoColor::Yellow, UnoFace::Number(2), 0),
        card(UnoColor::Green, UnoFace::Number(3), 0),
        card(UnoColor::Blue, UnoFace::Number(4), 0),
        card(UnoColor::Red, UnoFace::Number(5), 0),
        card(UnoColor::Yellow, UnoFace::Number(6), 0),
    ];
    game.players[0].hand = vec![pass, markers[0]];
    for (index, marker) in markers.iter().copied().enumerate().skip(1) {
        game.players[index].hand = vec![marker];
        game.uno_declared[index] = true;
        game.uno_exposed[index] = true;
    }
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(9), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), pass, None).unwrap();

    for (source, marker) in markers.iter().copied().enumerate() {
        let target = (source + 1) % markers.len();
        assert_eq!(game.player(UnoPlayerId(target)).unwrap().hand(), &[marker]);
    }
    assert!(game.uno_declared_players().next().is_none());
    assert!(game.uno_exposed_players().next().is_none());
    assert_eq!(
        game.pending_swap(),
        Some(PendingSwap::ChooseColor {
            player: UnoPlayerId(0)
        })
    );
}

#[test]
fn no_mercy_zero_pass_skips_eliminated_players() {
    let zero = card(UnoColor::Red, UnoFace::Number(0), 0);
    let markers = [
        card(UnoColor::Blue, UnoFace::Number(1), 0),
        card(UnoColor::Green, UnoFace::Number(2), 0),
        card(UnoColor::Yellow, UnoFace::Number(3), 0),
    ];
    let mut game = no_mercy_game(4);
    game.players[0].hand = vec![zero, markers[0]];
    game.players[1].hand = vec![markers[1]];
    game.players[2].hand.clear();
    game.players[2].eliminated = true;
    game.players[3].hand = vec![markers[2]];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), zero, None).unwrap();

    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand(), &[markers[2]]);
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand(), &[markers[0]]);
    assert!(game.player(UnoPlayerId(2)).unwrap().hand().is_empty());
    assert_eq!(game.player(UnoPlayerId(3)).unwrap().hand(), &[markers[1]]);
}

#[test]
fn hand_trade_targets_cannot_be_eliminated() {
    let mut game = swap_pack_game();
    game.pending_swap = Some(PendingSwapState::ForceTrade {
        player: UnoPlayerId(0),
    });
    game.current_player = UnoPlayerId(0);
    game.players[2].hand.clear();
    game.players[2].eliminated = true;

    assert_eq!(
        game.force_trade_hands(UnoPlayerId(0), UnoPlayerId(1), UnoPlayerId(2)),
        Err(GameError::InvalidSwapTargets)
    );
}

#[test]
fn last_swap_pack_card_runs_its_effect_before_winning() {
    let refresh = card(UnoColor::Red, UnoFace::RefreshHand, 0);
    let top = card(UnoColor::Red, UnoFace::Number(5), 0);
    let mut game = swap_pack_game();
    game.players[0].hand = vec![refresh];
    game.discard_pile = vec![top];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.play_card(UnoPlayerId(0), refresh, None),
        Ok(ActionOutcome::Played {
            effect: Some(PlayedEffect::HandRefreshed { count: 0 }),
            ..
        })
    ));
    assert_eq!(game.discard_pile(), &[top, refresh]);
    assert_eq!(game.pending_swap(), None);
    assert!(matches!(
        game.phase(),
        Phase::Finished(GameResult {
            winner: UnoPlayerId(0),
            ..
        })
    ));
}

#[test]
fn swap_pack_starting_cards_do_not_run_effects_and_wild_still_chooses_color() {
    let rules = UnoRuleSet {
        swap_pack: true,
        ..UnoRuleSet::default()
    };
    let colored = card(UnoColor::Red, UnoFace::SwapOne, 0);
    let mut deck = build_deck_for_rules(rules);
    let index = deck.iter().position(|card| *card == colored).unwrap();
    deck.swap(42, index);
    let game = GameState::new_with_deck(rules, 6, deck).unwrap();
    assert_eq!(game.top_card(), colored);
    assert_eq!(game.current_color(), Some(UnoColor::Red));
    assert_eq!(game.pending_swap(), None);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(0));

    let wild = UnoCard::wild(UnoFace::WildForceTrade, 0);
    let mut deck = build_deck_for_rules(rules);
    let index = deck.iter().position(|card| *card == wild).unwrap();
    deck.swap(42, index);
    let mut game = GameState::new_with_deck(rules, 6, deck).unwrap();
    assert_eq!(game.top_card(), wild);
    assert_eq!(game.current_color(), None);
    assert_eq!(game.pending_swap(), None);
    game.choose_initial_color(UnoPlayerId(0), UnoColor::Blue)
        .unwrap();
    assert_eq!(game.current_color(), Some(UnoColor::Blue));
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(0));
}

#[test]
fn no_mercy_draw_stack_only_accepts_an_equal_or_higher_value() {
    let draw_four = card(UnoColor::Red, UnoFace::DrawFour, 0);
    let draw_ten = UnoCard::wild(UnoFace::WildDrawTen, 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![draw_four, draw_ten];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 6;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(6));
    game.pending_draw_source = Some(UnoPlayerId(2));

    assert!(!game.can_play(UnoPlayerId(0), draw_four));
    assert!(game.can_play(UnoPlayerId(0), draw_ten));
}

#[test]
fn no_mercy_seven_swaps_the_actors_hand_with_one_target() {
    let seven = card(UnoColor::Red, UnoFace::Number(7), 0);
    let own = card(UnoColor::Blue, UnoFace::Number(1), 0);
    let target = card(UnoColor::Green, UnoFace::Number(2), 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![seven, own];
    game.players[1].hand = vec![target];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    game.play_card(UnoPlayerId(0), seven, None).unwrap();
    assert_eq!(
        game.pending_swap(),
        Some(PendingSwap::SevenSwap {
            player: UnoPlayerId(0)
        })
    );
    game.choose_seven_swap_target(UnoPlayerId(0), UnoPlayerId(1))
        .unwrap();
    assert_eq!(game.player(UnoPlayerId(0)).unwrap().hand(), &[target]);
    assert_eq!(game.player(UnoPlayerId(1)).unwrap().hand(), &[own]);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn no_mercy_draws_until_a_playable_card() {
    let miss = card(UnoColor::Blue, UnoFace::Number(2), 0);
    let playable = card(UnoColor::Red, UnoFace::Number(3), 0);
    let mut game = no_mercy_game(3);
    game.players[0].hand = vec![card(UnoColor::Green, UnoFace::Number(1), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.draw_pile = [miss, playable].into();
    game.current_color = Some(UnoColor::Red);
    game.current_player = UnoPlayerId(0);

    assert!(matches!(
        game.draw_card(UnoPlayerId(0)),
        Ok(ActionOutcome::DrewCards {
            cards,
            playable: Some(card),
            next_player: UnoPlayerId(0),
            ..
        }) if cards == vec![miss, playable] && card == playable
    ));
    assert_eq!(
        game.pass_after_draw(UnoPlayerId(0)),
        Err(GameError::MustPlayDrawnCard(playable))
    );
}

#[test]
fn no_mercy_eliminates_a_player_at_twenty_five_cards() {
    let mut game = no_mercy_game(3);
    game.players[0].hand = build_no_mercy_deck().into_iter().take(24).collect();
    game.current_player = UnoPlayerId(0);
    game.pending_draw = 1;
    game.pending_kind = Some(UnoPendingDrawKind::NoMercy(2));
    game.pending_draw_source = Some(UnoPlayerId(2));

    game.accept_draw_penalty(UnoPlayerId(0)).unwrap();
    assert!(game.player(UnoPlayerId(0)).unwrap().eliminated());
    assert!(game.player(UnoPlayerId(0)).unwrap().hand().is_empty());
    assert_eq!(game.elimination_order, vec![UnoPlayerId(0)]);
    assert_eq!(game.turn().unwrap().current_player, UnoPlayerId(1));
}

#[test]
fn no_mercy_eliminations_take_last_places_without_hand_scores() {
    let mut game = no_mercy_game(5);
    game.players[0].hand.clear();
    game.players[1].eliminated = true;
    game.players[1].hand.clear();
    game.elimination_order.push(UnoPlayerId(1));
    game.players[3].eliminated = true;
    game.players[3].hand.clear();
    game.elimination_order.push(UnoPlayerId(3));
    game.players[2].hand = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.players[4].hand = vec![card(UnoColor::Red, UnoFace::DrawTwo, 0)];

    let result = game.finish(UnoPlayerId(0));

    assert_eq!(result.hand_scores, vec![0, 0, 5, 0, 20]);
    assert_eq!(result.placements, vec![1, 5, 2, 4, 3]);
    assert_eq!(result.reference_deltas, vec![4, -3, 1, -2, 0]);
}
