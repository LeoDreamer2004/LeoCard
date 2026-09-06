use super::*;

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
fn color_roulette_keeps_the_uno_reaction_window_open() {
    fn prepared_game() -> GameState {
        let mut game = no_mercy_game(3);
        game.players[0].hand = vec![
            UnoCard::wild(UnoFace::WildColorRoulette, 0),
            card(UnoColor::Blue, UnoFace::Number(3), 0),
        ];
        game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
        game.current_color = Some(UnoColor::Red);
        game.current_player = UnoPlayerId(0);
        game
    }

    let mut recover = prepared_game();
    recover
        .play_card(
            UnoPlayerId(0),
            UnoCard::wild(UnoFace::WildColorRoulette, 0),
            None,
        )
        .unwrap();
    assert_eq!(
        recover.uno_exposed_players().collect::<Vec<_>>(),
        vec![UnoPlayerId(0)]
    );
    assert!(recover.can_call_uno(UnoPlayerId(0)));
    assert!(matches!(
        recover.call_uno(UnoPlayerId(0)),
        Ok(ActionOutcome::UnoCalled {
            player: UnoPlayerId(0)
        })
    ));

    let mut report = prepared_game();
    report
        .play_card(
            UnoPlayerId(0),
            UnoCard::wild(UnoFace::WildColorRoulette, 0),
            None,
        )
        .unwrap();
    assert!(matches!(
        report.report_uno(UnoPlayerId(2), UnoPlayerId(0)),
        Ok(ActionOutcome::UnoReported {
            reporter: UnoPlayerId(2),
            target: UnoPlayerId(0),
            ref cards,
        }) if cards.len() == 2
    ));
    assert_eq!(
        report.pending_swap(),
        Some(PendingSwap::ColorRoulette {
            player: UnoPlayerId(1)
        })
    );
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
