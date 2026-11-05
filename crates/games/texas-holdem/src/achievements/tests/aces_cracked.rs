use super::aces_cracked_helpers::{amount, c, play};
use crate::TexasAction::{AllIn, Call, Check};
use crate::TexasRank::{Ace, Eight, Jack, King, Nine, Queen, Ten, Three, Two};
use crate::TexasSuit::{Club, Diamond, Heart, Spade};
use crate::{TexasCard, TexasHandCategory, TexasPhase};
use leocard_game_common::PlayerId;

const AA: [TexasCard; 2] = [c(Ace, Spade), c(Ace, Heart)];
const KQ: [TexasCard; 2] = [c(King, Spade), c(Queen, Heart)];
const TWO_PAIR_BOARD: [TexasCard; 5] = [
    c(King, Club),
    c(Queen, Diamond),
    c(Eight, Heart),
    c(Two, Club),
    c(Three, Diamond),
];

#[test]
fn preflop_aces_cracked_by_suited_or_offsuit_king_queen_awards_silver() {
    for queen_suit in [Heart, Spade] {
        for actions in [
            vec![(0, AllIn), (1, Call)],
            vec![(0, Call), (1, AllIn), (0, Call)],
        ] {
            let game = play(
                &[AA, [c(King, Spade), c(Queen, queen_suit)]],
                TWO_PAIR_BOARD,
                vec![12, 12],
                &actions,
            );
            let report = game.hand_statistics(PlayerId(0)).unwrap();
            assert_eq!(report.hole_cards, AA);
            assert!(report.showdown && report.preflop_all_in);
            assert!(report.lost_to_lower_unpaired_hand);
            assert_eq!(report.won_chips, 0);
            let winner = game.hand_statistics(PlayerId(1)).unwrap();
            assert_eq!(winner.category, Some(TexasHandCategory::TwoPair));
            assert_eq!(winner.won_chips, 24);
            assert_eq!(amount(report), 1, "{queen_suit:?}, {actions:?}");
        }
    }
}

#[test]
fn nonqualifying_showdowns_do_not_award_aces_cracked() {
    for (name, own, opponent, board, after_flop, lost, won) in [
        (
            "lost to a pocket pair",
            AA,
            [c(King, Spade), c(King, Heart)],
            TWO_PAIR_BOARD,
            false,
            false,
            0,
        ),
        (
            "lost to unpaired AK containing an ace",
            AA,
            [c(Ace, Club), c(King, Spade)],
            [
                c(King, Club),
                c(King, Diamond),
                c(Queen, Diamond),
                c(Eight, Heart),
                c(Two, Club),
            ],
            false,
            false,
            0,
        ),
        (
            "board plays for a tie",
            AA,
            KQ,
            [
                c(Ten, Club),
                c(Jack, Club),
                c(Queen, Club),
                c(King, Club),
                c(Ace, Club),
            ],
            false,
            false,
            12,
        ),
        (
            "aces win",
            AA,
            KQ,
            [
                c(Two, Club),
                c(Three, Diamond),
                c(Eight, Heart),
                c(Nine, Club),
                c(Jack, Diamond),
            ],
            false,
            false,
            24,
        ),
        (
            "all in only after the flop",
            AA,
            KQ,
            TWO_PAIR_BOARD,
            true,
            true,
            0,
        ),
        (
            "not pocket aces",
            [c(Jack, Spade), c(Jack, Heart)],
            KQ,
            TWO_PAIR_BOARD,
            false,
            true,
            0,
        ),
    ] {
        let actions = if after_flop {
            vec![(0, Call), (1, Check), (1, AllIn), (0, Call)]
        } else {
            vec![(0, AllIn), (1, Call)]
        };
        let game = play(&[own, opponent], board, vec![12, 12], &actions);
        let report = game.hand_statistics(PlayerId(0)).unwrap();
        assert_eq!(report.hole_cards, own, "{name}");
        assert!(report.showdown, "{name}");
        assert_eq!(report.preflop_all_in, !after_flop, "{name}");
        assert_eq!(report.lost_to_lower_unpaired_hand, lost, "{name}");
        assert_eq!(report.won_chips, won, "{name}");
        assert_eq!(report.tied_winners, if won == 12 { 2 } else { 0 }, "{name}");
        assert_eq!(amount(report), 0, "{name}");
    }
}

#[test]
fn unshared_side_pot_winner_does_not_crack_short_stack_aces() {
    let game = play(
        &[AA, KQ, [c(Jack, Spade), c(Jack, Heart)]],
        [
            c(Ace, Diamond),
            c(King, Club),
            c(Queen, Diamond),
            c(Eight, Heart),
            c(Two, Club),
        ],
        vec![4, 12, 12],
        &[(0, AllIn), (1, AllIn), (2, Call)],
    );
    let TexasPhase::Complete(result) = game.phase() else {
        panic!("expected settlement");
    };
    assert_eq!(
        result
            .awards
            .iter()
            .map(|pot| (pot.amount, pot.winners.clone()))
            .collect::<Vec<_>>(),
        vec![(12, vec![PlayerId(0)]), (16, vec![PlayerId(1)])],
    );
    let report = game.hand_statistics(PlayerId(0)).unwrap();
    assert!(report.showdown && report.preflop_all_in);
    assert_eq!(report.category, Some(TexasHandCategory::ThreeOfAKind));
    assert_eq!(report.won_chips, 12);
    assert_eq!(report.won_side_pot_chips, 0);
    assert!(!report.lost_to_lower_unpaired_hand);
    assert_eq!(amount(report), 0);
    let side_winner = game.hand_statistics(PlayerId(1)).unwrap();
    assert_eq!(side_winner.category, Some(TexasHandCategory::TwoPair));
    assert_eq!(side_winner.won_side_pot_chips, 16);
    assert!(
        game.hand_statistics(PlayerId(2))
            .unwrap()
            .lost_to_lower_unpaired_hand
    );
}
