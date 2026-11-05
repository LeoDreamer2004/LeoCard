use super::aces_cracked_helpers::{amount, c, play};
use crate::TexasAction::{AllIn, Call, Check};
use crate::TexasRank::{Ace, Eight, Jack, King, Nine, Queen, Ten, Three, Two};
use crate::TexasSuit::{Club, Diamond, Heart, Spade};
use crate::{TexasCard, TexasPhase};
use leocard_game_common::PlayerId;

const AA: [TexasCard; 2] = [c(Ace, Spade), c(Ace, Heart)];
const KQ: [TexasCard; 2] = [c(King, Spade), c(Queen, Heart)];
const KK: [TexasCard; 2] = [c(King, Diamond), c(King, Heart)];
const JJ: [TexasCard; 2] = [c(Jack, Spade), c(Jack, Heart)];
const BOARD: [TexasCard; 5] = [
    c(King, Club),
    c(Queen, Diamond),
    c(Eight, Heart),
    c(Two, Club),
    c(Three, Diamond),
];

#[test]
fn only_actual_winning_hole_cards_can_crack_aces() {
    for (holes, board, winners) in [
        // KQ beats AA, but KK actually wins the pot.
        (vec![AA, KQ, KK], BOARD, vec![PlayerId(2)]),
        (
            vec![AA, [c(Ace, Club), c(Ace, Diamond)]],
            BOARD,
            vec![PlayerId(1), PlayerId(0)],
        ),
        // Opposing AA wins with a flush, but is still a paired starting hand.
        (
            vec![AA, [c(Ace, Club), c(Ace, Diamond)]],
            [
                c(King, Club),
                c(Queen, Club),
                c(Eight, Club),
                c(Two, Club),
                c(Three, Diamond),
            ],
            vec![PlayerId(1)],
        ),
    ] {
        let actions = if holes.len() == 3 {
            vec![(0, AllIn), (1, Call), (2, Call)]
        } else {
            vec![(0, AllIn), (1, Call)]
        };
        let stacks = vec![12; holes.len()];
        let game = play(&holes, board, stacks, &actions);
        let TexasPhase::Complete(result) = game.phase() else {
            panic!("expected settlement");
        };
        assert_eq!(result.awards.len(), 1);
        assert_eq!(result.awards[0].winners, winners);
        let report = game.hand_statistics(PlayerId(0)).unwrap();
        assert!(report.showdown && report.preflop_all_in);
        assert!(!report.lost_to_lower_unpaired_hand);
        assert_eq!(amount(report), 0);
    }
}

#[test]
fn aces_cracked_uses_eligibility_for_each_actual_pot() {
    for (name, holes, stacks, actions, awards, lost, won) in [
        (
            "eligible main pot loss counts despite winning a side pot",
            vec![AA, KQ, JJ],
            vec![12, 4, 12],
            vec![(0, AllIn), (1, Call), (2, Call)],
            vec![(12, vec![PlayerId(1)]), (16, vec![PlayerId(0)])],
            true,
            16,
        ),
        (
            "eligible side pot loss counts despite paired main pot winner",
            vec![AA, KQ, KK],
            vec![12, 12, 4],
            vec![(0, AllIn), (1, Call), (2, Call)],
            vec![(12, vec![PlayerId(2)]), (16, vec![PlayerId(1)])],
            true,
            0,
        ),
        (
            "ineligible stronger side pot winner does not count",
            vec![AA, KQ, KK, JJ],
            vec![4, 12, 4, 12],
            vec![(3, AllIn), (0, Call), (1, Call), (2, Call)],
            vec![(16, vec![PlayerId(2)]), (16, vec![PlayerId(1)])],
            false,
            0,
        ),
    ] {
        let game = play(&holes, BOARD, stacks, &actions);
        let TexasPhase::Complete(result) = game.phase() else {
            panic!("expected settlement");
        };
        assert_eq!(
            result
                .awards
                .iter()
                .map(|pot| (pot.amount, pot.winners.clone()))
                .collect::<Vec<_>>(),
            awards,
            "{name}",
        );
        let report = game.hand_statistics(PlayerId(0)).unwrap();
        assert!(report.showdown && report.preflop_all_in, "{name}");
        assert_eq!(report.lost_to_lower_unpaired_hand, lost, "{name}");
        assert_eq!(report.won_chips, won, "{name}");
        assert_eq!(amount(report), u64::from(lost), "{name}");
    }
}

#[test]
fn pocket_kings_losing_to_unpaired_hole_cards_do_not_qualify() {
    let game = play(
        &[KK, KQ],
        [
            c(Jack, Club),
            c(Ten, Diamond),
            c(Nine, Heart),
            c(Two, Club),
            c(Three, Diamond),
        ],
        vec![12, 12],
        &[(0, AllIn), (1, Call)],
    );
    let report = game.hand_statistics(PlayerId(0)).unwrap();
    assert!(report.showdown && report.preflop_all_in);
    assert!(report.lost_to_lower_unpaired_hand);
    assert_eq!(report.won_chips, 0);
    assert_eq!(amount(report), 0);
}

#[test]
fn forced_blind_all_in_is_not_a_voluntary_preflop_all_in() {
    let game = play(&[AA, KQ], BOARD, vec![1, 12], &[(1, Check)]);
    let report = game.hand_statistics(PlayerId(0)).unwrap();
    assert!(report.showdown && report.lost_to_lower_unpaired_hand);
    assert_eq!(report.won_chips, 0);
    assert!(!report.preflop_all_in);
    assert_eq!(amount(report), 0);
}
