use super::helpers::*;
use crate::{
    TexasAction, TexasCard, TexasGameState, TexasHandCategory, TexasRank, TexasRuleSet, TexasSuit,
};
use TexasRank::*;
use TexasSuit::*;
use leocard_game_common::PlayerId;

fn c(rank: TexasRank, suit: TexasSuit) -> TexasCard {
    TexasCard::new(suit, rank)
}

fn two_run_game(reverse_runs: bool, scoop: bool) -> TexasGameState {
    let holes = [
        c(King, Heart),
        c(Queen, Club),
        c(Eight, Heart),
        c(King, Diamond),
        c(Jack, Club),
        c(Seven, Heart),
    ];
    let losing = [
        c(Queen, Diamond),
        c(Jack, Diamond),
        c(Two, Spade),
        c(Three, Club),
        c(Four, Heart),
    ];
    let winning = if scoop {
        [
            c(Queen, Heart),
            c(Jack, Heart),
            c(Two, Club),
            c(Three, Diamond),
            c(Four, Spade),
        ]
    } else {
        [
            c(King, Spade),
            c(Nine, Spade),
            c(Six, Diamond),
            c(Five, Club),
            c(Two, Heart),
        ]
    };
    let mut prefix = holes.to_vec();
    prefix.extend(if reverse_runs { winning } else { losing });
    prefix.extend(if reverse_runs { losing } else { winning });
    let rules = TexasRuleSet {
        player_count: 3,
        squid_markers: 1,
        side_pots: false,
        run_it_twice: true,
        ..Default::default()
    };
    let mut game = TexasGameState::new_with_stacks(
        rules,
        PlayerId(2),
        vec![10, 10, 20],
        ordered_deck(&prefix),
    )
    .unwrap();
    blinds(&mut game);
    act(&mut game, TexasAction::Fold);
    act(&mut game, TexasAction::AllIn);
    act(&mut game, TexasAction::Call);
    game
}

#[test]
fn split_run_break_even_counts_one_hand_and_no_net_win_in_either_board_order() {
    for reverse in [false, true] {
        let mut game = two_run_game(reverse, false);
        let history = game.betting_history().to_vec();
        let last_action = game.last_action_statistics();
        vote_all(&mut game, true);
        assert_eq!(result(&game).final_stacks, [10, 10, 20]);
        assert_eq!(game.betting_history(), history);
        assert_eq!(game.last_action_statistics(), last_action);
        assert_eq!(result(&game).awards[0].amount, 10);
        assert_eq!(
            result(&game).second_run.as_ref().unwrap().awards[0].amount,
            10
        );
        for player in [PlayerId(0), PlayerId(1)] {
            let report = game.hand_statistics(player).unwrap();
            assert_eq!(
                report.hole_cards,
                game.players()[player.index()].hole_cards()
            );
            assert!(report.preflop_all_in);
            assert_eq!(report.won_chips, 0);
            assert_eq!(report.won_pot, 10);
            assert_eq!(report.won_side_pot_chips, 0);
            assert!(report.defeated_categories.is_empty());
            assert_eq!(report.match_statistics.hands, 1);
            assert_eq!(report.match_statistics.won_hands, 0);
        }
        assert_eq!(
            game.hand_statistics(PlayerId(0)).unwrap().category,
            Some(TexasHandCategory::ThreeOfAKind)
        );
        assert_eq!(
            game.hand_statistics(PlayerId(1)).unwrap().category,
            Some(TexasHandCategory::TwoPair)
        );
        assert!(game.squid().markers.iter().all(Option::is_none));
        assert!(game.squid().transfers.is_empty());
    }
}

#[test]
fn aggregate_poker_gain_excludes_squid_and_scoop_settles_once() {
    for reverse in [false, true] {
        let mut game = two_run_game(reverse, true);
        vote_all(&mut game, true);
        let loser = game.hand_statistics(PlayerId(0)).unwrap();
        assert_eq!(loser.won_chips, 0);
        assert_eq!(loser.won_pot, 0);
        assert_eq!(loser.match_statistics.hands, 1);
        assert_eq!(loser.match_statistics.won_hands, 0);
        let winner = game.hand_statistics(PlayerId(1)).unwrap();
        assert_eq!(winner.won_chips, 10); // 20 in poker receipts minus 10 retained investment
        assert_eq!(winner.won_pot, 20);
        assert_eq!(winner.won_side_pot_chips, 0);
        assert_eq!(winner.defeated_categories, [TexasHandCategory::OnePair]);
        assert_eq!(winner.defeated_pocket_pairs, 1);
        assert_eq!(winner.defeated_preflop_all_in_pairs, 1);
        assert_eq!(winner.match_statistics.hands, 1);
        assert_eq!(winner.match_statistics.won_hands, 1);
        assert_eq!(game.squid().markers, [Some(PlayerId(1))]);
        assert!(game.squid().settled);
        assert_eq!(game.squid().transfers.len(), 1); // C pays once; A is already broke
        assert_eq!(result(&game).final_stacks, [0, 22, 18]);
    }
}

#[test]
fn winning_a_tied_board_does_not_count_as_a_net_win_after_losing_the_other() {
    let holes = [
        c(King, Heart),
        c(Queen, Club),
        c(Eight, Heart),
        c(King, Diamond),
        c(Jack, Club),
        c(Seven, Heart),
    ];
    let tied = [Ten, Jack, Queen, King, Ace].map(|rank| c(rank, Spade));
    let decisive = [
        c(Queen, Diamond),
        c(Jack, Diamond),
        c(Two, Spade),
        c(Three, Club),
        c(Four, Heart),
    ];
    for reverse in [false, true] {
        let mut prefix = holes.to_vec();
        prefix.extend(if reverse { decisive } else { tied });
        prefix.extend(if reverse { tied } else { decisive });
        let rules = TexasRuleSet {
            player_count: 3,
            side_pots: false,
            run_it_twice: true,
            ..Default::default()
        };
        let mut game =
            TexasGameState::new_with_stacks(rules, PlayerId(2), vec![10; 3], ordered_deck(&prefix))
                .unwrap();
        blinds(&mut game);
        act(&mut game, TexasAction::AllIn);
        act(&mut game, TexasAction::Call);
        act(&mut game, TexasAction::Call);
        vote_all(&mut game, true);
        assert_eq!(result(&game).final_stacks, [5, 20, 5]);
        for player in [PlayerId(0), PlayerId(2)] {
            let report = game.hand_statistics(player).unwrap();
            assert_eq!(report.won_chips, 0); // 5 received is less than 10 invested
            assert_eq!(report.won_pot, 15);
            assert_eq!(report.tied_winners, 3);
            assert_eq!(report.match_statistics.hands, 1);
            assert_eq!(report.match_statistics.won_hands, 0);
        }
        let winner = game.hand_statistics(PlayerId(1)).unwrap();
        assert_eq!(winner.won_chips, 10);
        assert_eq!(winner.won_pot, 30);
        assert!(winner.defeated_categories.is_empty()); // tied opponents on one board
        assert_eq!(winner.match_statistics.hands, 1);
        assert_eq!(winner.match_statistics.won_hands, 1);
    }
}
