use super::*;
use crate::{TexasHoldemRank, TexasHoldemSuit};
use TexasHoldemSuit::{Club, Diamond, Heart, Spade};

fn ordered_deck(prefix: &[TexasHoldemCard], short_deck: bool) -> Vec<TexasHoldemCard> {
    let prefix_set = prefix.iter().copied().collect::<HashSet<_>>();
    let mut deck = prefix.to_vec();
    deck.extend(
        build_deck(short_deck)
            .into_iter()
            .filter(|card| !prefix_set.contains(card)),
    );
    deck
}

fn c(rank: TexasHoldemRank, suit: TexasHoldemSuit) -> TexasHoldemCard {
    TexasHoldemCard::new(suit, rank)
}

fn post_blinds(state: &mut GameState) {
    let small = state.current_player().expect("小盲应先行动");
    state.act(small, TexasHoldemAction::PostBlind).unwrap();
    let big = state.current_player().expect("大盲应随后行动");
    state.act(big, TexasHoldemAction::PostBlind).unwrap();
}

#[test]
fn dealer_blinds_and_preflop_action_follow_clockwise_order() {
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    assert_eq!(state.small_blind(), TexasHoldemPlayerId(1));
    assert_eq!(state.big_blind(), TexasHoldemPlayerId(2));
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(1)));
    assert_eq!(state.players()[0].stack(), 20);
    assert_eq!(state.players()[1].stack(), 20);
    assert_eq!(state.players()[2].stack(), 20);
    assert_eq!(
        state.blind_to_post(),
        Some((TexasHoldemPlayerId(1), TexasHoldemBlindKind::Small, 1))
    );
    assert!(matches!(
        state.act(TexasHoldemPlayerId(1), TexasHoldemAction::Call),
        Err(GameError::MustPostBlind)
    ));
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::PostBlind)
        .unwrap();
    assert_eq!(
        state.blind_to_post(),
        Some((TexasHoldemPlayerId(2), TexasHoldemBlindKind::Big, 2))
    );
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::PostBlind)
        .unwrap();
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(0)));
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
    let state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            omaha: true,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
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
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Call)
        .unwrap();
    let flop = state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Check)
        .unwrap();
    assert!(matches!(
        flop,
        ActionOutcome::StreetAdvanced {
            street: TexasHoldemStreet::Flop,
            community_cards: 3,
            ..
        }
    ));
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(1)));

    for (street, expected_cards) in [(TexasHoldemStreet::Turn, 4), (TexasHoldemStreet::River, 5)] {
        state
            .act(TexasHoldemPlayerId(1), TexasHoldemAction::Check)
            .unwrap();
        state
            .act(TexasHoldemPlayerId(2), TexasHoldemAction::Check)
            .unwrap();
        let outcome = state
            .act(TexasHoldemPlayerId(0), TexasHoldemAction::Check)
            .unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::StreetAdvanced { street: actual, .. } if actual == street
        ));
        assert_eq!(state.community().len(), expected_cards);
    }
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Check)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Check)
        .unwrap();
    assert!(matches!(
        state
            .act(TexasHoldemPlayerId(0), TexasHoldemAction::Check)
            .unwrap(),
        ActionOutcome::HandComplete(_)
    ));
}

#[test]
fn all_in_side_pots_are_awarded_independently() {
    let prefix = [
        c(TexasHoldemRank::King, Spade),
        c(TexasHoldemRank::Queen, Spade),
        c(TexasHoldemRank::Ace, Spade),
        c(TexasHoldemRank::King, Heart),
        c(TexasHoldemRank::Queen, Heart),
        c(TexasHoldemRank::Ace, Heart),
        c(TexasHoldemRank::Two, Club),
        c(TexasHoldemRank::Three, Diamond),
        c(TexasHoldemRank::Seven, Spade),
        c(TexasHoldemRank::Eight, Club),
        c(TexasHoldemRank::Nine, Diamond),
    ];
    let mut state = GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        vec![5, 10, 20],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::AllIn)
        .unwrap();
    let result = match state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Call)
        .unwrap()
    {
        ActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };
    assert_eq!(result.awards.len(), 2);
    assert_eq!(result.awards[0].amount, 15);
    assert_eq!(result.awards[0].winners, vec![TexasHoldemPlayerId(0)]);
    assert_eq!(result.awards[1].amount, 10);
    assert_eq!(result.awards[1].winners, vec![TexasHoldemPlayerId(1)]);
    assert_eq!(result.final_stacks, vec![15, 10, 10]);
}

#[test]
fn ignore_kickers_splits_a_showdown_between_equal_made_hands() {
    // 三位玩家分别组成 AAQQK、AAQQJ、AAQQT；开启规则后踢脚牌不参与比较。
    let prefix = [
        c(TexasHoldemRank::King, Spade),
        c(TexasHoldemRank::Jack, Spade),
        c(TexasHoldemRank::Ten, Spade),
        c(TexasHoldemRank::Three, Heart),
        c(TexasHoldemRank::Four, Heart),
        c(TexasHoldemRank::Five, Heart),
        c(TexasHoldemRank::Ace, Club),
        c(TexasHoldemRank::Ace, Diamond),
        c(TexasHoldemRank::Queen, Club),
        c(TexasHoldemRank::Queen, Diamond),
        c(TexasHoldemRank::Two, Spade),
    ];
    let mut state = GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: 3,
            ignore_kickers: true,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        vec![5, 5, 5],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::AllIn)
        .unwrap();
    let result = match state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Call)
        .unwrap()
    {
        ActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };

    assert_eq!(result.awards.len(), 1);
    assert_eq!(result.awards[0].amount, 15);
    assert_eq!(
        result.awards[0].winners,
        vec![
            TexasHoldemPlayerId(1),
            TexasHoldemPlayerId(2),
            TexasHoldemPlayerId(0)
        ]
    );
    assert_eq!(result.final_stacks, vec![5, 5, 5]);
}

#[test]
fn several_distinct_all_ins_create_independently_eligible_side_pots() {
    // 每个较短筹码玩家都拿到比后续玩家更大的口袋对子，因此能够验证：
    // 他只参与不超过自己投入额的底池，不能赢走更深层的边池。
    let prefix = [
        c(TexasHoldemRank::King, Spade),
        c(TexasHoldemRank::Queen, Spade),
        c(TexasHoldemRank::Jack, Spade),
        c(TexasHoldemRank::Ace, Spade),
        c(TexasHoldemRank::King, Heart),
        c(TexasHoldemRank::Queen, Heart),
        c(TexasHoldemRank::Jack, Heart),
        c(TexasHoldemRank::Ace, Heart),
        c(TexasHoldemRank::Two, Club),
        c(TexasHoldemRank::Three, Diamond),
        c(TexasHoldemRank::Seven, Spade),
        c(TexasHoldemRank::Eight, Club),
        c(TexasHoldemRank::Nine, Diamond),
    ];
    let mut state = GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: 4,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        vec![5, 10, 15, 20],
        ordered_deck(&prefix, false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(3), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::AllIn)
        .unwrap();
    let result = match state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::AllIn)
        .unwrap()
    {
        ActionOutcome::HandComplete(result) => result,
        other => panic!("expected showdown, got {other:?}"),
    };

    assert_eq!(
        result
            .awards
            .iter()
            .map(|award| (award.amount, award.winners.clone()))
            .collect::<Vec<_>>(),
        vec![
            (20, vec![TexasHoldemPlayerId(0)]),
            (15, vec![TexasHoldemPlayerId(1)]),
            (10, vec![TexasHoldemPlayerId(2)]),
            (5, vec![TexasHoldemPlayerId(3)]),
        ]
    );
    assert_eq!(result.final_stacks, vec![20, 15, 10, 5]);
}

#[test]
fn folding_everyone_else_ends_the_hand_without_showdown() {
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Fold)
        .unwrap();
    let result = match state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Fold)
        .unwrap()
    {
        ActionOutcome::HandComplete(result) => result,
        other => panic!("expected immediate win, got {other:?}"),
    };
    assert!(!result.showdown);
    assert_eq!(result.awards[0].winners, vec![TexasHoldemPlayerId(2)]);
    assert_eq!(result.final_stacks.iter().sum::<u32>(), 60);
}

#[test]
fn rejected_action_never_mutates_the_authoritative_state() {
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    let before = state.clone();
    assert!(matches!(
        state.act(TexasHoldemPlayerId(0), TexasHoldemAction::Check),
        Err(GameError::CannotCheckWhileFacingBet { .. })
    ));
    assert_eq!(state, before);
}

#[test]
fn full_raise_reopens_action_and_enforces_the_minimum_increment() {
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    assert!(matches!(
        state.act(TexasHoldemPlayerId(0), TexasHoldemAction::RaiseTo(3)),
        Err(GameError::RaiseBelowMinimum {
            minimum_target: 4,
            ..
        })
    ));
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::RaiseTo(4))
        .unwrap();
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Call)
        .unwrap();
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(0)));
    assert!(matches!(
        state
            .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
            .unwrap(),
        ActionOutcome::StreetAdvanced {
            street: TexasHoldemStreet::Flop,
            ..
        }
    ));
}

#[test]
fn checking_keeps_raise_right_against_a_short_all_in_opening_bet() {
    let mut state = GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        vec![20, 20, 3],
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);

    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Check)
        .unwrap();

    // 翻牌圈最低完整下注为 2；玩家 2 只剩 1，因此这是不足额的首次下注。
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Check)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(1)));

    // 玩家 1 之前只是 check，并未面对下注，仍可完成一次加注到 3。
    assert!(
        state
            .act(TexasHoldemPlayerId(1), TexasHoldemAction::RaiseTo(3))
            .is_ok()
    );
}

#[test]
fn short_all_in_does_not_reopen_the_original_raisers_action() {
    let mut state = GameState::new_with_stacks(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        vec![20, 20, 5],
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);

    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Call)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::Check)
        .unwrap();

    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::RaiseTo(2))
        .unwrap();
    // 玩家 2 只剩 3：加到 3 的增量小于最低完整加注幅度 2。
    state
        .act(TexasHoldemPlayerId(2), TexasHoldemAction::AllIn)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Call)
        .unwrap();
    assert_eq!(state.current_player(), Some(TexasHoldemPlayerId(1)));
    assert!(matches!(
        state.act(TexasHoldemPlayerId(1), TexasHoldemAction::RaiseTo(5)),
        Err(GameError::RaiseNotReopened)
    ));
}

#[test]
fn dealer_button_moves_to_the_next_funded_player() {
    let mut state = GameState::new_with_deck(
        TexasHoldemRuleSet {
            player_count: 3,
            ..TexasHoldemRuleSet::default()
        },
        TexasHoldemPlayerId(0),
        build_deck(false),
    )
    .unwrap();
    post_blinds(&mut state);
    state
        .act(TexasHoldemPlayerId(0), TexasHoldemAction::Fold)
        .unwrap();
    state
        .act(TexasHoldemPlayerId(1), TexasHoldemAction::Fold)
        .unwrap();
    state.start_next_hand(build_deck(false)).unwrap();
    assert_eq!(state.dealer(), TexasHoldemPlayerId(1));
    assert_eq!(state.hand_number(), 1);
}
