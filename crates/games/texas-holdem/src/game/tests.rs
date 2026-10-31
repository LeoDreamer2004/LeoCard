use super::*;
use crate::{TexasCard, TexasRuleSet, build_deck};
use crate::{TexasRank, TexasSuit};
use TexasSuit::{Club, Diamond, Heart, Spade};
use leocard_game_common::PlayerId;

use std::collections::HashSet;

fn ordered_deck(prefix: &[TexasCard], short_deck: bool) -> Vec<TexasCard> {
    let prefix_set = prefix.iter().copied().collect::<HashSet<_>>();
    let mut deck = prefix.to_vec();
    deck.extend(
        build_deck(short_deck)
            .into_iter()
            .filter(|card| !prefix_set.contains(card)),
    );
    deck
}

fn c(rank: TexasRank, suit: TexasSuit) -> TexasCard {
    TexasCard::new(suit, rank)
}

fn post_blinds(state: &mut TexasGameState) {
    let small = state.current_player().expect("小盲应先行动");
    state.act(small, TexasAction::PostBlind).unwrap();
    let big = state.current_player().expect("大盲应随后行动");
    state.act(big, TexasAction::PostBlind).unwrap();
}

#[test]
fn dealer_blinds_and_preflop_action_follow_clockwise_order() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    assert_eq!(state.small_blind(), PlayerId(1));
    assert_eq!(state.big_blind(), PlayerId(2));
    assert_eq!(state.current_player(), Some(PlayerId(1)));
    assert_eq!(state.players()[0].stack(), 20);
    assert_eq!(state.players()[1].stack(), 20);
    assert_eq!(state.players()[2].stack(), 20);
    assert_eq!(
        state.blind_to_post(),
        Some((PlayerId(1), TexasBlindKind::Small, 1))
    );
    assert!(matches!(
        state.act(PlayerId(1), TexasAction::Call),
        Err(TexasGameError::MustPostBlind)
    ));
    state.act(PlayerId(1), TexasAction::PostBlind).unwrap();
    assert_eq!(
        state.blind_to_post(),
        Some((PlayerId(2), TexasBlindKind::Big, 2))
    );
    state.act(PlayerId(2), TexasAction::PostBlind).unwrap();
    assert_eq!(state.current_player(), Some(PlayerId(0)));
    assert_eq!(state.players()[1].stack(), 19);
    assert_eq!(state.players()[2].stack(), 18);
    assert!(
        state
            .players()
            .iter()
            .all(|player| player.hole_cards().len() == 2)
    );
}

#[test]
fn omaha_deals_four_private_cards_to_every_funded_player() {
    let state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            omaha: true,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();

    assert!(
        state
            .players()
            .iter()
            .all(|player| player.hole_cards().len() == 4)
    );
    assert_eq!(state.draw_pile_len(), 52 - 3 * 4);
}

#[test]
fn four_betting_rounds_deal_exactly_five_community_cards() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(0), TexasAction::Call).unwrap();
    state.act(PlayerId(1), TexasAction::Call).unwrap();
    let flop = state.act(PlayerId(2), TexasAction::Check).unwrap();
    assert!(matches!(
        flop,
        TexasActionOutcome::StreetAdvanced {
            street: TexasStreet::Flop,
            community_cards: 3,
            ..
        }
    ));
    assert_eq!(state.current_player(), Some(PlayerId(1)));

    for (street, expected_cards) in [(TexasStreet::Turn, 4), (TexasStreet::River, 5)] {
        state.act(PlayerId(1), TexasAction::Check).unwrap();
        state.act(PlayerId(2), TexasAction::Check).unwrap();
        let outcome = state.act(PlayerId(0), TexasAction::Check).unwrap();
        assert!(matches!(
            outcome,
            TexasActionOutcome::StreetAdvanced { street: actual, .. } if actual == street
        ));
        assert_eq!(state.community().len(), expected_cards);
    }
    state.act(PlayerId(1), TexasAction::Check).unwrap();
    state.act(PlayerId(2), TexasAction::Check).unwrap();
    assert!(matches!(
        state.act(PlayerId(0), TexasAction::Check).unwrap(),
        TexasActionOutcome::HandComplete(_)
    ));
}

#[test]
fn all_in_side_pots_are_awarded_independently() {
    let prefix = [
        c(TexasRank::King, Spade),
        c(TexasRank::Queen, Spade),
        c(TexasRank::Ace, Spade),
        c(TexasRank::King, Heart),
        c(TexasRank::Queen, Heart),
        c(TexasRank::Ace, Heart),
        c(TexasRank::Two, Club),
        c(TexasRank::Three, Diamond),
        c(TexasRank::Seven, Spade),
        c(TexasRank::Eight, Club),
        c(TexasRank::Nine, Diamond),
    ];
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        vec![5, 10, 20],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(0), TexasAction::AllIn).unwrap();
    state.act(PlayerId(1), TexasAction::AllIn).unwrap();
    let result = match state.act(PlayerId(2), TexasAction::Call).unwrap() {
        TexasActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };
    assert_eq!(result.awards.len(), 2);
    assert_eq!(result.awards[0].amount, 15);
    assert_eq!(result.awards[0].winners, vec![PlayerId(0)]);
    assert_eq!(result.awards[1].amount, 10);
    assert_eq!(result.awards[1].winners, vec![PlayerId(1)]);
    assert_eq!(result.final_stacks, vec![15, 10, 10]);
}

#[test]
fn ignore_kickers_splits_a_showdown_between_equal_made_hands() {
    // 三位玩家分别组成 AAQQK、AAQQJ、AAQQT；开启规则后踢脚牌不参与比较。
    let prefix = [
        c(TexasRank::King, Spade),
        c(TexasRank::Jack, Spade),
        c(TexasRank::Ten, Spade),
        c(TexasRank::Three, Heart),
        c(TexasRank::Four, Heart),
        c(TexasRank::Five, Heart),
        c(TexasRank::Ace, Club),
        c(TexasRank::Ace, Diamond),
        c(TexasRank::Queen, Club),
        c(TexasRank::Queen, Diamond),
        c(TexasRank::Two, Spade),
    ];
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            ignore_kickers: true,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        vec![5, 5, 5],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(0), TexasAction::AllIn).unwrap();
    state.act(PlayerId(1), TexasAction::AllIn).unwrap();
    let result = match state.act(PlayerId(2), TexasAction::Call).unwrap() {
        TexasActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };

    assert_eq!(result.awards.len(), 1);
    assert_eq!(result.awards[0].amount, 15);
    assert_eq!(
        result.awards[0].winners,
        vec![PlayerId(1), PlayerId(2), PlayerId(0)]
    );
    assert_eq!(result.final_stacks, vec![5, 5, 5]);
}

#[test]
fn several_distinct_all_ins_create_independently_eligible_side_pots() {
    // 每个较短筹码玩家都拿到比后续玩家更大的口袋对子，因此能够验证：
    // 他只参与不超过自己投入额的底池，不能赢走更深层的边池。
    let prefix = [
        c(TexasRank::King, Spade),
        c(TexasRank::Queen, Spade),
        c(TexasRank::Jack, Spade),
        c(TexasRank::Ace, Spade),
        c(TexasRank::King, Heart),
        c(TexasRank::Queen, Heart),
        c(TexasRank::Jack, Heart),
        c(TexasRank::Ace, Heart),
        c(TexasRank::Two, Club),
        c(TexasRank::Three, Diamond),
        c(TexasRank::Seven, Spade),
        c(TexasRank::Eight, Club),
        c(TexasRank::Nine, Diamond),
    ];
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 4,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        vec![5, 10, 15, 20],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(3), TexasAction::AllIn).unwrap();
    state.act(PlayerId(0), TexasAction::AllIn).unwrap();
    state.act(PlayerId(1), TexasAction::AllIn).unwrap();
    let result = match state.act(PlayerId(2), TexasAction::AllIn).unwrap() {
        TexasActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };

    assert_eq!(
        result
            .awards
            .iter()
            .map(|award| (award.amount, award.winners.clone()))
            .collect::<Vec<_>>(),
        vec![
            (20, vec![PlayerId(0)]),
            (15, vec![PlayerId(1)]),
            (10, vec![PlayerId(2)]),
            (5, vec![PlayerId(3)]),
        ]
    );
    assert_eq!(result.final_stacks, vec![20, 15, 10, 5]);
}

#[test]
fn folding_everyone_else_ends_the_hand_without_showdown() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(0), TexasAction::Fold).unwrap();
    let result = match state.act(PlayerId(1), TexasAction::Fold).unwrap() {
        TexasActionOutcome::HandComplete(result) => result,
        other => panic!("expected immediate win, got {other:?}"),
    };
    assert!(!result.showdown);
    assert_eq!(result.awards[0].winners, vec![PlayerId(2)]);
    assert_eq!(result.final_stacks.iter().sum::<u32>(), 60);
}

#[test]
fn rejected_action_never_mutates_the_authoritative_state() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    let before = state.clone();
    assert!(matches!(
        state.act(PlayerId(0), TexasAction::Check),
        Err(TexasGameError::CannotCheckWhileFacingBet { .. })
    ));
    assert_eq!(state, before);
}

#[test]
fn full_raise_reopens_action_and_enforces_the_minimum_increment() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    assert!(matches!(
        state.act(PlayerId(0), TexasAction::RaiseTo(3)),
        Err(TexasGameError::RaiseBelowMinimum {
            minimum_target: 4,
            ..
        })
    ));
    let before = state.clone();
    assert_eq!(
        state.act(PlayerId(0), TexasAction::RaiseTo(5)),
        Err(TexasGameError::RaiseInvalidIncrement {
            increment: 2,
            target: 5,
        })
    );
    assert_eq!(state, before);
    state.act(PlayerId(0), TexasAction::Call).unwrap();
    let mut larger_raise = state.clone();
    larger_raise
        .act(PlayerId(1), TexasAction::RaiseTo(6))
        .unwrap();
    assert_eq!(larger_raise.minimum_raise(), 4);
    assert_eq!(
        larger_raise.act(PlayerId(2), TexasAction::RaiseTo(11)),
        Err(TexasGameError::RaiseInvalidIncrement {
            increment: 4,
            target: 11,
        })
    );
    larger_raise
        .act(PlayerId(2), TexasAction::RaiseTo(10))
        .unwrap();
    state.act(PlayerId(1), TexasAction::RaiseTo(4)).unwrap();
    state.act(PlayerId(2), TexasAction::Call).unwrap();
    assert_eq!(state.current_player(), Some(PlayerId(0)));
    assert!(matches!(
        state.act(PlayerId(0), TexasAction::Call).unwrap(),
        TexasActionOutcome::StreetAdvanced {
            street: TexasStreet::Flop,
            ..
        }
    ));
}

#[test]
fn checking_keeps_raise_right_against_a_short_all_in_opening_bet() {
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        vec![20, 20, 3],
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);

    state.act(PlayerId(0), TexasAction::Call).unwrap();
    state.act(PlayerId(1), TexasAction::Call).unwrap();
    state.act(PlayerId(2), TexasAction::Check).unwrap();

    // 翻牌圈最低完整下注为 2；玩家 2 只剩 1，因此这是不足额的首次下注。
    state.act(PlayerId(1), TexasAction::Check).unwrap();
    state.act(PlayerId(2), TexasAction::AllIn).unwrap();
    state.act(PlayerId(0), TexasAction::Call).unwrap();
    assert_eq!(state.current_player(), Some(PlayerId(1)));

    // 玩家 1 之前只是 check，并未面对下注，仍可完成一次加注到 3。
    assert!(state.act(PlayerId(1), TexasAction::RaiseTo(3)).is_ok());
}

#[test]
fn short_all_in_does_not_reopen_the_original_raisers_action() {
    let mut state = TexasGameState::new_with_stacks(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        vec![20, 20, 5],
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);

    state.act(PlayerId(0), TexasAction::Call).unwrap();
    state.act(PlayerId(1), TexasAction::Call).unwrap();
    state.act(PlayerId(2), TexasAction::Check).unwrap();

    state.act(PlayerId(1), TexasAction::RaiseTo(2)).unwrap();
    // 玩家 2 只剩 3：加到 3 的增量小于最低完整加注幅度 2。
    state.act(PlayerId(2), TexasAction::AllIn).unwrap();
    state.act(PlayerId(0), TexasAction::Call).unwrap();
    assert_eq!(state.current_player(), Some(PlayerId(1)));
    assert!(matches!(
        state.act(PlayerId(1), TexasAction::RaiseTo(5)),
        Err(TexasGameError::RaiseNotReopened)
    ));
}

#[test]
fn dealer_button_moves_to_the_next_funded_player() {
    let mut state = TexasGameState::new_with_deck(
        TexasRuleSet {
            player_count: 3,
            ..TexasRuleSet::default()
        },
        PlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state.act(PlayerId(0), TexasAction::Fold).unwrap();
    state.act(PlayerId(1), TexasAction::Fold).unwrap();
    state.start_next_hand(build_deck(false)).unwrap();
    assert_eq!(state.dealer(), PlayerId(1));
    assert_eq!(state.hand_number(), 1);
}

#[test]
fn expanded_tables_finish_a_full_board_without_exhausting_the_deck() {
    for player_count in 6..=8 {
        for short_deck in [false, true] {
            for omaha in [false, true] {
                let rules = TexasRuleSet {
                    player_count,
                    short_deck,
                    omaha,
                    ..TexasRuleSet::default()
                };
                let game =
                    TexasGameState::new_with_deck(rules, PlayerId(0), build_deck(short_deck));
                if player_count > 6 && short_deck && omaha {
                    assert!(matches!(game, Err(TexasGameError::InvalidRules(_))));
                    continue;
                }
                let mut game = game.unwrap();
                for player in game.players() {
                    assert_eq!(player.hole_cards().len(), rules.hole_card_count());
                }
                while let Some(player) = game.current_player() {
                    let action = if game.blind_to_post().is_some() {
                        TexasAction::PostBlind
                    } else {
                        TexasAction::AllIn
                    };
                    game.act(player, action).unwrap();
                }
                assert_eq!(game.community().len(), 5);
                assert_eq!(
                    game.players()
                        .iter()
                        .map(|player| player.stack())
                        .sum::<u32>(),
                    u32::from(player_count) * u32::from(rules.starting_chips)
                );
            }
        }
    }
}
