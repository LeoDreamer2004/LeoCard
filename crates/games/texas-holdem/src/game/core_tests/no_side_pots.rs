use super::helpers::*;
use crate::{
    TexasAction, TexasActionOutcome, TexasCard, TexasGameError, TexasGameState, TexasPhase,
    TexasRank, TexasRuleSet, TexasStreet, TexasSuit, build_deck,
};
use leocard_game_common::PlayerId;

#[test]
fn seven_player_example_refunds_only_after_every_response_and_keeps_paid_history() {
    let board = [
        TexasRank::Ten,
        TexasRank::Jack,
        TexasRank::Queen,
        TexasRank::King,
        TexasRank::Ace,
    ]
    .map(|rank| TexasCard::new(TexasSuit::Spade, rank));
    let mut prefix = build_deck(false)
        .into_iter()
        .filter(|card| !board.contains(card))
        .take(14)
        .collect::<Vec<_>>();
    prefix.extend(board);
    let stacks = [100, 260, 70, 40, 85, 200, 30];
    let rules = TexasRuleSet {
        player_count: 7,
        side_pots: false,
        run_it_twice: true,
        ..Default::default()
    };
    let mut game =
        TexasGameState::new_with_stacks(rules, PlayerId(6), stacks.to_vec(), ordered_deck(&prefix))
            .unwrap();
    to_street(&mut game, TexasStreet::Flop, 20);
    for action in [
        TexasAction::AllIn,
        TexasAction::Call,
        TexasAction::AllIn,
        TexasAction::AllIn,
        TexasAction::AllIn,
        TexasAction::Call,
    ] {
        act(&mut game, action);
        assert_eq!(game.current_bet(), 80);
    }
    assert_eq!(
        game.players()
            .iter()
            .map(|p| p.committed_total())
            .collect::<Vec<_>>(),
        [100, 100, 70, 40, 85, 100, 20]
    );
    assert_eq!(game.players()[4].stack(), 0); // E shoves ALL 65 despite D's later 20 cap
    assert_eq!(game.current_player(), Some(PlayerId(6)));
    assert!(matches!(
        act(&mut game, TexasAction::Fold),
        TexasActionOutcome::RunoutVoteStarted {
            current_player: PlayerId(0)
        }
    ));
    assert_eq!(game.pot(), 260);
    assert_eq!(
        game.players()
            .iter()
            .map(|p| p.committed_total())
            .collect::<Vec<_>>(),
        [40, 40, 40, 40, 40, 40, 20]
    );
    assert_eq!(
        game.players().iter().map(|p| p.stack()).collect::<Vec<_>>(),
        [60, 220, 30, 0, 45, 160, 10]
    );
    assert_eq!(
        game.betting_history()
            .iter()
            .rev()
            .take(7)
            .map(|r| r.payment)
            .collect::<Vec<_>>(),
        [0, 80, 65, 20, 50, 80, 80]
    );
    vote_all(&mut game, false);
    let result = result(&game);
    assert_eq!(result.awards.len(), 1);
    assert_eq!(result.awards[0].amount, 260);
    assert_eq!(
        result.awards[0].winners,
        (0..6).map(PlayerId).collect::<Vec<_>>()
    );
    assert_eq!(
        result.final_stacks.iter().sum::<u32>(),
        stacks.iter().sum::<u32>()
    );
}

#[test]
fn locked_short_call_spends_all_and_final_long_call_records_gross_payment_despite_refund() {
    let mut game = game(&[82, 22, 67, 202], false);
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::AllIn); // A80 locks the target
    act(&mut game, TexasAction::Call); // B20 reduces only the eventual refund cap
    act(&mut game, TexasAction::Call); // C MUST pay all65, not20
    assert_eq!(game.players()[2].stack(), 0);
    assert_eq!(game.players()[2].committed_total(), 67);
    assert_eq!(game.amount_to_call(PlayerId(3)).unwrap(), 80);
    assert!(matches!(
        act(&mut game, TexasAction::Call),
        TexasActionOutcome::HandComplete(_)
    ));
    assert_eq!(game.pot(), 88);
    assert!(game.players().iter().all(|p| p.committed_total() == 22));
    assert_eq!(game.betting_history().last().unwrap().payment, 80);
    assert!(!game.last_action_statistics().unwrap().raised);
    assert_eq!(result(&game).awards.len(), 1);
    assert!(result(&game).second_run.is_none());
    assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 373);
}

#[test]
fn undercall_locks_own_target_and_rejects_long_stack_raises_atomically() {
    let mut game = game(&[22, 100, 100, 100], true);
    to_street(&mut game, TexasStreet::Flop, 2);
    // A checks, B raises, C calls, D calls, A's 20 is the first all-in undercall.
    act(&mut game, TexasAction::Check);
    act(&mut game, TexasAction::RaiseTo(40));
    act(&mut game, TexasAction::Call);
    act(&mut game, TexasAction::Call);
    act(&mut game, TexasAction::AllIn);
    assert_eq!(game.current_bet(), 20);
    assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
    assert_eq!(game.pot(), 88);
    assert!(game.players().iter().all(|p| p.committed_total() == 22));

    let mut game = super::helpers::game(&[22, 100, 100, 100], true);
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::Check);
    act(&mut game, TexasAction::RaiseTo(40));
    act(&mut game, TexasAction::Fold);
    act(&mut game, TexasAction::RaiseTo(80));
    act(&mut game, TexasAction::AllIn);
    assert_eq!(game.current_bet(), 20);
    assert_eq!(game.current_player(), Some(PlayerId(1)));
    assert_eq!(game.amount_to_call(PlayerId(1)).unwrap(), 0);
    // Earlier investments are retained until all responses are complete.
    assert_eq!(game.players()[1].committed_street(), 40);
    assert_eq!(game.players()[3].committed_street(), 80);
    for action in [TexasAction::RaiseTo(160), TexasAction::AllIn] {
        let before = game.clone();
        assert_eq!(
            game.act(PlayerId(1), action),
            Err(TexasGameError::RaiseNotReopened)
        );
        assert_eq!(game, before);
    }
    assert!(!game.raise_allowed(PlayerId(1)).unwrap());
    assert!(!game.all_in_allowed(PlayerId(1)).unwrap());
    act(&mut game, TexasAction::Check);
    assert_eq!(game.pot(), 68); // A/B/D at22 plus C's folded2
}

#[test]
fn first_undercall_lets_later_long_stack_call_without_all_in_statistics() {
    let mut game = game(&[202, 22, 52, 12, 102], true);
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::RaiseTo(80)); // B's earlier investment
    act(&mut game, TexasAction::AllIn); // A20, not the old bet80, fixes the target
    assert_eq!(game.current_bet(), 20);
    assert_eq!(game.current_player(), Some(PlayerId(2)));
    assert_eq!(game.amount_to_call(PlayerId(2)).unwrap(), 20);
    assert!(!game.all_in_allowed(PlayerId(2)).unwrap());
    let before = game.clone();
    assert_eq!(
        game.act(PlayerId(2), TexasAction::AllIn),
        Err(TexasGameError::RaiseNotReopened)
    );
    assert_eq!(game, before);
    act(&mut game, TexasAction::Call); // C50 is long relative to A20
    assert_eq!(game.players()[2].stack(), 30);
    assert_eq!(game.players()[2].committed_street(), 20);
    assert!(!game.players()[2].all_in());
    let statistics = game.last_action_statistics().unwrap();
    assert_eq!(statistics.all_in_amount, 0);
    assert!(!statistics.raised);
    assert!(!statistics.full_raise);
    assert_eq!(game.betting_history().last().unwrap().payment, 20);
    assert_eq!(game.players()[0].committed_street(), 80); // no early refund

    act(&mut game, TexasAction::AllIn); // D10 only reduces the final refund cap
    assert_eq!(game.current_bet(), 20);
    assert_eq!(game.amount_to_call(PlayerId(4)).unwrap(), 20);
    assert_eq!(game.players()[0].committed_street(), 80);
    act(&mut game, TexasAction::Call);
    assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
    assert_eq!(game.pot(), 60);
    assert!(game.players().iter().all(|p| p.committed_total() == 12));
    assert_eq!(game.betting_history().last().unwrap().payment, 20);
    vote_all(&mut game, false);
    assert_eq!(result(&game).awards.len(), 1);
    assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 390);
}

#[test]
fn first_all_in_raise_uses_old_bet_for_full_raise_detection_and_history() {
    for (target, full_raise, minimum_raise, bet_level) in [(60, false, 40, 1), (120, true, 80, 2)] {
        let mut game = game(&[202, target + 2, 202], true);
        to_street(&mut game, TexasStreet::Flop, 2);
        act(&mut game, TexasAction::RaiseTo(40));
        act(&mut game, TexasAction::AllIn);
        assert_eq!(game.current_bet(), target);
        assert_eq!(game.minimum_raise(), minimum_raise);
        let statistics = game.last_action_statistics().unwrap();
        assert!(statistics.raised);
        assert_eq!(statistics.full_raise, full_raise);
        assert_eq!(statistics.bet_level, bet_level);
        assert_eq!(statistics.all_in_amount, target);
        assert_eq!(game.betting_history().last().unwrap().payment, target);
        assert!(!game.raise_allowed(PlayerId(2)).unwrap());
        act(&mut game, TexasAction::Call);
        act(&mut game, TexasAction::Call);
        assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
        assert_eq!(game.pot(), 3 * (target + 2));
    }
}

#[test]
fn folded_excess_stays_dead_and_folded_short_players_never_set_cap() {
    let mut game = game(&[100, 100, 22, 100, 100], true);
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::RaiseTo(40));
    act(&mut game, TexasAction::RaiseTo(80));
    act(&mut game, TexasAction::AllIn); // C undercalls20, locking its own20
    assert_eq!(game.current_bet(), 20);
    assert_eq!(game.players()[0].committed_street(), 40);
    assert_eq!(game.players()[1].committed_street(), 80);
    act(&mut game, TexasAction::Call);
    assert_eq!(game.players()[3].committed_street(), 20);
    act(&mut game, TexasAction::Fold); // folded2 is below cap and ignored
    act(&mut game, TexasAction::Fold); // A's folded42 is ABOVE cap and not refunded
    assert_eq!(game.pot(), 110); // dead42 + B/C/D22 each + dead2
    assert_eq!(game.players()[0].committed_total(), 42);
    assert_eq!(game.players()[3].stack(), 78);
    vote_all(&mut game, false);
    assert_eq!(result(&game).awards.len(), 1);
    assert!(
        result(&game).awards[0]
            .winners
            .iter()
            .all(|id| [PlayerId(1), PlayerId(2), PlayerId(3)].contains(id))
    );
}

#[test]
fn blind_exhaustion_and_call_to_zero_close_betting_after_responses() {
    for stacks in [[1, 50, 50], [50, 1, 50], [50, 2, 50]] {
        let mut game = game(&stacks, true);
        blinds(&mut game);
        let target = if stacks[1] == 2 { 2 } else { 1 };
        assert_eq!(game.current_bet(), target);
        while matches!(game.phase(), TexasPhase::Betting(_)) {
            let player = game.current_player().unwrap();
            assert!(!game.raise_allowed(player).unwrap());
            let action = if game.amount_to_call(player).unwrap() == 0 {
                TexasAction::Check
            } else {
                TexasAction::Call
            };
            act(&mut game, action);
        }
        assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
        assert_eq!(game.pot(), target * 3);
    }
    let mut game = game(&[6, 100, 100], true);
    to_street(&mut game, TexasStreet::Flop, 2);
    act(&mut game, TexasAction::Check);
    act(&mut game, TexasAction::RaiseTo(4));
    act(&mut game, TexasAction::Call);
    act(&mut game, TexasAction::Call); // zero stack via call rather than AllIn
    assert_eq!(game.phase(), &TexasPhase::ChoosingRunout);
    assert_eq!(game.pot(), 18);
    assert_eq!(game.last_action_statistics().unwrap().all_in_amount, 4);
}

#[test]
fn uncontested_all_in_refunds_before_awarding_and_never_votes() {
    let mut game = game(&[100, 100, 20], true);
    blinds(&mut game);
    act(&mut game, TexasAction::AllIn); // C first, total20
    act(&mut game, TexasAction::Fold);
    act(&mut game, TexasAction::Fold);
    assert!(!result(&game).showdown);
    assert!(game.runout_votes().is_empty());
    assert_eq!(result(&game).awards[0].amount, 23);
    assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 220);
}

#[test]
fn fixed_limit_omaha_and_short_deck_share_the_lock_and_single_pot_path() {
    for (fixed_limit, omaha, short_deck) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, true, true),
    ] {
        let rules = TexasRuleSet {
            player_count: 3,
            fixed_limit,
            omaha,
            short_deck,
            side_pots: false,
            run_it_twice: true,
            ..Default::default()
        };
        let mut game = TexasGameState::new_with_stacks(
            rules,
            PlayerId(2),
            vec![4, 9, 9],
            build_deck(short_deck),
        )
        .unwrap();
        to_street(&mut game, TexasStreet::Flop, 2);
        act(&mut game, TexasAction::AllIn);
        assert!(!game.raise_allowed(PlayerId(1)).unwrap());
        act(&mut game, TexasAction::Call);
        act(&mut game, TexasAction::Call);
        assert_eq!(game.pot(), 12);
        vote_all(&mut game, true);
        assert_eq!(result(&game).awards.len(), 1);
        assert_eq!(result(&game).second_run.as_ref().unwrap().awards.len(), 1);
        assert_eq!(result(&game).final_stacks.iter().sum::<u32>(), 22);
    }
}
