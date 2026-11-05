use super::helpers::*;
use crate::{
    TexasAction, TexasActionOutcome, TexasBotObservation, TexasGameError, TexasGameState,
    TexasPassiveAutoPlay, TexasPhase, TexasRuleSet, TexasStreet, build_deck,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[test]
fn unanimous_votes_preserve_shared_prefix_and_draw_each_remaining_card_once() {
    for (street, prefix_len) in [
        (TexasStreet::PreFlop, 0),
        (TexasStreet::Flop, 3),
        (TexasStreet::Turn, 4),
    ] {
        let stacks = if prefix_len == 0 {
            [30, 30, 5]
        } else {
            [5, 30, 30]
        };
        let mut game = game(&stacks, true);
        to_street(&mut game, street, 2);
        let prefix = game.community().to_vec();
        act(&mut game, TexasAction::AllIn);
        act(&mut game, TexasAction::Call);
        let outcome = act(&mut game, TexasAction::Call);
        assert_eq!(
            outcome,
            TexasActionOutcome::RunoutVoteStarted {
                current_player: PlayerId(0)
            }
        );
        assert_eq!(game.pot(), 15);
        let history = game.betting_history().to_vec();
        for voter in [PlayerId(0), PlayerId(1)] {
            assert_eq!(game.current_player(), Some(voter));
            assert!(
                matches!(game.choose_runout(voter, true).unwrap(), TexasActionOutcome::RunoutChosen { player, twice: true, .. } if player == voter)
            );
        }
        assert!(matches!(
            game.choose_runout(PlayerId(2), true).unwrap(),
            TexasActionOutcome::HandComplete(_)
        ));
        assert_eq!(game.betting_history(), history);
        let result = result(&game);
        let second = result.second_run.as_ref().unwrap();
        assert_eq!(&result.community[..prefix_len], prefix);
        assert_eq!(&second.community[..prefix_len], prefix);
        let deck = build_deck(false);
        assert_eq!(&result.community[prefix_len..], &deck[6 + prefix_len..11]);
        assert_eq!(&second.community[prefix_len..], &deck[11..16 - prefix_len]);
        let cards = game
            .players()
            .iter()
            .flat_map(|p| p.hole_cards())
            .copied()
            .chain(result.community.iter().copied())
            .chain(second.community[prefix_len..].iter().copied())
            .collect::<Vec<_>>();
        assert_eq!(
            cards.iter().copied().collect::<HashSet<_>>().len(),
            cards.len()
        );
        assert_eq!(result.awards[0].amount, 8);
        assert_eq!(second.awards[0].amount, 7);
        assert_eq!(result.final_stacks.iter().sum::<u32>(), 65);
        assert_eq!(game.draw_pile_len(), 52 - cards.len());
        for id in [PlayerId(0), PlayerId(1), PlayerId(2)] {
            assert_eq!(game.hand_statistics(id).unwrap().match_statistics.hands, 1);
        }
        game.start_next_hand(build_deck(false)).unwrap();
        assert!(game.runout_votes().is_empty());
        assert!(game.betting_history().is_empty());
        blinds(&mut game);
        let current = game.current_player().unwrap();
        assert!(game.raise_allowed(current).unwrap());
    }
}

#[test]
fn once_vote_still_collects_every_live_voter_and_errors_are_atomic() {
    let mut game = game(&[5, 30, 30, 30], true);
    assert_eq!(
        game.choose_runout(PlayerId(0), false),
        Err(TexasGameError::NotChoosingRunout)
    );
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::AllIn);
    act(&mut game, TexasAction::Fold);
    act(&mut game, TexasAction::Call);
    act(&mut game, TexasAction::Call);
    let before = game.clone();
    assert_eq!(
        game.choose_runout(PlayerId(2), true),
        Err(TexasGameError::NotPlayersTurn {
            expected: PlayerId(0),
            actual: PlayerId(2)
        })
    );
    assert_eq!(game, before);
    assert_eq!(
        game.choose_runout(PlayerId(99), true),
        Err(TexasGameError::InvalidPlayer(PlayerId(99)))
    );
    assert_eq!(game, before);
    assert_eq!(
        game.act(PlayerId(0), TexasAction::Check),
        Err(TexasGameError::MustChooseRunout)
    );
    assert_eq!(game, before);
    assert!(TexasPassiveAutoPlay::choose_for_game(&game, PlayerId(0)).is_none());
    assert!(TexasBotObservation::from_game(&game, PlayerId(0)).is_none());
    assert_eq!(
        game.choose_runout(PlayerId(0), false).unwrap(),
        TexasActionOutcome::RunoutChosen {
            player: PlayerId(0),
            twice: false,
            next_player: PlayerId(2)
        }
    );
    assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
    let before = game.clone();
    assert_eq!(
        game.choose_runout(PlayerId(0), true),
        Err(TexasGameError::RunoutAlreadyChosen(PlayerId(0)))
    );
    assert_eq!(game, before);
    assert_eq!(
        game.choose_runout(PlayerId(2), true).unwrap(),
        TexasActionOutcome::RunoutChosen {
            player: PlayerId(2),
            twice: true,
            next_player: PlayerId(3)
        }
    );
    game.choose_runout(PlayerId(3), true).unwrap();
    assert_eq!(
        game.runout_votes(),
        [
            (PlayerId(0), false),
            (PlayerId(2), true),
            (PlayerId(3), true)
        ]
    );
    assert!(result(&game).second_run.is_none());
    assert_eq!(result(&game).awards[0].amount, 17); // folded B's2 stays in the pot
    assert_eq!(game.draw_pile_len(), 52 - 8 - 5);
    let before = game.clone();
    assert_eq!(
        game.choose_runout(PlayerId(0), true),
        Err(TexasGameError::NotChoosingRunout)
    );
    assert_eq!(game, before);
}

#[test]
fn river_all_in_and_normal_river_showdown_never_offer_runout_voting() {
    for all_in in [true, false] {
        let mut game = game(&[5, 30, 30], true);
        to_street(&mut game, TexasStreet::River, 2);
        assert!(game.runout_votes().is_empty());
        act(
            &mut game,
            if all_in {
                TexasAction::AllIn
            } else {
                TexasAction::Check
            },
        );
        act(
            &mut game,
            if all_in {
                TexasAction::Call
            } else {
                TexasAction::Check
            },
        );
        assert!(matches!(
            act(
                &mut game,
                if all_in {
                    TexasAction::Call
                } else {
                    TexasAction::Check
                }
            ),
            TexasActionOutcome::HandComplete(_)
        ));
        assert!(result(&game).second_run.is_none());
        assert!(game.runout_votes().is_empty());
        assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 65);
    }
}

#[test]
fn voters_wrap_clockwise_and_include_the_dealer_last() {
    let rules = TexasRuleSet {
        player_count: 4,
        side_pots: false,
        run_it_twice: true,
        ..Default::default()
    };
    let mut game =
        TexasGameState::new_with_stacks(rules, PlayerId(1), vec![30, 30, 5, 30], build_deck(false))
            .unwrap();
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::AllIn);
    for _ in 0..3 {
        act(&mut game, TexasAction::Call);
    }
    for voter in [PlayerId(2), PlayerId(3), PlayerId(0), PlayerId(1)] {
        assert_eq!(game.current_player(), Some(voter));
        game.choose_runout(voter, true).unwrap();
    }
    assert!(result(&game).second_run.is_some());
}

#[test]
fn exhausted_heads_up_blinds_start_voting_without_a_betting_action() {
    let rules = TexasRuleSet {
        player_count: 3,
        side_pots: false,
        run_it_twice: true,
        ..Default::default()
    };
    let mut game =
        TexasGameState::new_with_stacks(rules, PlayerId(0), vec![1, 1, 0], build_deck(false))
            .unwrap();
    act(&mut game, TexasAction::PostBlind);
    assert_eq!(
        act(&mut game, TexasAction::PostBlind),
        TexasActionOutcome::RunoutVoteStarted {
            current_player: PlayerId(1)
        }
    );
    assert_eq!(game.pot(), 2);
    assert_eq!(
        game.choose_runout(PlayerId(1), true).unwrap(),
        TexasActionOutcome::RunoutChosen {
            player: PlayerId(1),
            twice: true,
            next_player: PlayerId(0)
        }
    );
    game.choose_runout(PlayerId(0), true).unwrap();
    assert_eq!(result(&game).awards[0].amount, 1);
    assert_eq!(
        result(&game).second_run.as_ref().unwrap().awards[0].amount,
        1
    );
    assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 2);
    assert_eq!(game.betting_history().len(), 2);
    assert_eq!(
        game.hand_statistics(PlayerId(0))
            .unwrap()
            .match_statistics
            .hands,
        1
    );
}

#[test]
fn every_voter_position_can_force_one_run() {
    for once_voter in 0..3 {
        let mut game = game(&[5, 30, 30], true);
        to_street(&mut game, TexasStreet::Flop, 2);
        act(&mut game, TexasAction::AllIn);
        act(&mut game, TexasAction::Call);
        act(&mut game, TexasAction::Call);
        for voter in 0..3 {
            game.choose_runout(PlayerId(voter), voter != once_voter)
                .unwrap();
        }
        assert!(result(&game).second_run.is_none());
        assert_eq!(game.runout_votes().len(), 3);
    }
}
