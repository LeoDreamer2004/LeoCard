use crate::{UnoCard, UnoColor, UnoFace, UnoRuleSet, build_deck_for_rules};
use leocard_game_common::PlayerId;

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
    let mut game = UnoGameState::new_with_deck(UnoRuleSet::default(), 6, deck).unwrap();
    game.players[0].hand = vec![wild_draw_four];
    let target_hand = game.players[1].hand.len();
    let outcome = game
        .play_card(PlayerId(0), wild_draw_four, Some(UnoColor::Blue))
        .unwrap();
    assert!(matches!(outcome, UnoActionOutcome::Played { .. }));
    let turn = game
        .turn()
        .expect("the final draw penalty still needs resolving");
    assert_eq!(turn.current_color, Some(UnoColor::Blue));
    assert_eq!(turn.current_player, PlayerId(1));
    assert_eq!(turn.pending_draw, 4);
    assert_eq!(turn.challenge_offender, Some(PlayerId(0)));

    assert!(matches!(
        game.accept_draw_penalty(PlayerId(1)),
        Ok(UnoActionOutcome::PenaltyDrawn { ref cards, .. }) if cards.len() == 4
    ));
    assert_eq!(game.players[1].hand.len(), target_hand + 4);
    assert!(matches!(
        game.phase(),
        UnoPhase::Finished(UnoGameResult {
            winner: PlayerId(0),
            ..
        })
    ));
    assert_eq!(
        game.challenge_draw_four(PlayerId(1)),
        Err(UnoGameError::GameAlreadyFinished)
    );
}

#[test]
fn last_skip_finishes_after_the_target_loses_their_turn() {
    let skip = card(UnoColor::Red, UnoFace::Skip, 0);
    let mut game = UnoGameState::new_with_deck(UnoRuleSet::default(), 6, build_deck()).unwrap();
    game.players[0].hand = vec![skip];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);

    assert!(matches!(
        game.play_card(PlayerId(0), skip, None),
        Ok(UnoActionOutcome::Played { .. })
    ));
    assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));
    assert!(matches!(game.phase(), UnoPhase::Playing));

    assert!(matches!(
        game.resolve_skip(PlayerId(1)),
        Ok(UnoActionOutcome::SkipResolved {
            player: PlayerId(1),
            remaining: 0,
            ..
        })
    ));
    assert!(matches!(
        game.phase(),
        UnoPhase::Finished(UnoGameResult {
            winner: PlayerId(0),
            ..
        })
    ));
}

#[test]
fn final_skip_chain_finishes_after_acceptance_without_waiting_for_future_turns() {
    for skip_draw_penalty in [false, true] {
        for face in [UnoFace::Skip, UnoFace::ReverseSkip] {
            let rules = UnoRuleSet {
                action_stacking: true,
                reverse_pack: face == UnoFace::ReverseSkip,
                skip_draw_penalty,
                uno_callout: false,
                ..UnoRuleSet::default()
            };
            let mut game =
                UnoGameState::new_with_deck(rules, 3, build_deck_for_rules(rules)).unwrap();
            let first = card(UnoColor::Red, face, 0);
            let following = card(UnoColor::Yellow, UnoFace::Skip, 0);
            game.players[0].hand = vec![first];
            game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
            game.current_color = Some(UnoColor::Red);
            game.current_player = PlayerId(0);
            game.direction = UnoDirection::Clockwise;
            game.play_card(PlayerId(0), first, None).unwrap();
            let follower = game.turn().unwrap().current_player;
            game.players[follower.index()].hand =
                vec![following, card(UnoColor::Blue, UnoFace::Number(3), 0)];
            game.play_card(follower, following, None).unwrap();
            let target = game.turn().unwrap().current_player;
            assert_ne!(target, PlayerId(0));
            assert_eq!(game.turn().unwrap().pending_skip, 2);
            let before = game.player(target).unwrap().hand().len();
            let outcome = game.resolve_skip(target).unwrap();
            assert!(matches!(
                outcome,
                UnoActionOutcome::SkipResolved { remaining: 1, .. }
            ));
            assert_eq!(
                game.player(target).unwrap().hand().len(),
                before + usize::from(skip_draw_penalty)
            );
            assert!(matches!(
                game.phase(),
                UnoPhase::Finished(UnoGameResult {
                    winner: PlayerId(0),
                    ..
                })
            ));
            assert!(
                game.players
                    .iter()
                    .all(|player| game.skipped_turns(player.id) == Some(0))
            );
            assert_eq!(
                game.draw_card(target),
                Err(UnoGameError::GameAlreadyFinished)
            );
        }
    }
}

#[test]
fn returning_skip_penalty_refills_the_finisher_instead_of_awarding_a_win() {
    let rules = UnoRuleSet {
        action_stacking: true,
        skip_draw_penalty: true,
        uno_callout: false,
        ..UnoRuleSet::default()
    };
    let mut game = UnoGameState::new_with_deck(rules, 2, build_deck()).unwrap();
    let first = card(UnoColor::Red, UnoFace::Skip, 0);
    let following = card(UnoColor::Yellow, UnoFace::Skip, 0);
    game.players[0].hand = vec![first];
    game.players[1].hand = vec![following, card(UnoColor::Blue, UnoFace::Number(3), 0)];
    game.discard_pile = vec![card(UnoColor::Red, UnoFace::Number(5), 0)];
    game.current_color = Some(UnoColor::Red);
    game.current_player = PlayerId(0);
    game.play_card(PlayerId(0), first, None).unwrap();
    game.play_card(PlayerId(1), following, None).unwrap();
    game.resolve_skip(PlayerId(0)).unwrap();
    assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), 1);
    assert!(matches!(game.phase(), UnoPhase::Playing));
}
