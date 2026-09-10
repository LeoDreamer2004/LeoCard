use super::*;
use crate::{QiGuiCard, QiGuiRank, QiGuiRuleSet, QiGuiSuit, build_deck, can_beat, classify};
use std::collections::HashSet;

fn deck_with_prefix(prefix: &[QiGuiCard], deck_count: u8) -> Vec<QiGuiCard> {
    let prefix_set: HashSet<_> = prefix.iter().copied().collect();
    let mut deck = prefix.to_vec();
    deck.extend(
        build_deck(deck_count)
            .into_iter()
            .filter(|card| !prefix_set.contains(card)),
    );
    deck
}

#[test]
fn deals_clockwise_and_exposes_lowest_card_owner() {
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let diamond_four = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four);
    let deck = deck_with_prefix(
        &[
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Ace),
            diamond_four,
            QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Six),
        ],
        1,
    );
    let game = GameState::new_with_deck(rules, deck).unwrap();

    assert_eq!(game.starting_card().player, QiGuiPlayerId(1));
    assert_eq!(game.starting_card().card, diamond_four);
    assert_eq!(game.trick().unwrap().current_player(), QiGuiPlayerId(1));
    assert!(game.players().iter().all(|player| player.hand().len() == 5));
    assert_eq!(game.draw_pile_len(), 39);
}

#[test]
fn duplicate_lowest_cards_use_the_shuffled_deal_order_as_tie_breaker() {
    let rules = QiGuiRuleSet {
        deck_count: 2,
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let first_diamond_four = QiGuiCard::suited(1, QiGuiSuit::Diamond, QiGuiRank::Four);
    let later_diamond_four = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four);
    let deck = deck_with_prefix(
        &[
            QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Six),
            QiGuiCard::suited(0, QiGuiSuit::Spade, QiGuiRank::Six),
            first_diamond_four,
            later_diamond_four,
        ],
        2,
    );
    let game = GameState::new_with_deck(rules, deck).unwrap();

    // 玩家 2 先从洗好的牌序中拿到最小牌，所以不按较小座位号选择玩家 0。
    assert_eq!(game.starting_card().player, QiGuiPlayerId(2));
    assert_eq!(game.starting_card().card, first_diamond_four);
}

#[test]
fn winner_collects_points_then_everyone_refills() {
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let diamond_four = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four);
    let diamond_five = QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Five);
    let club_five = QiGuiCard::suited(0, QiGuiSuit::Club, QiGuiRank::Five);
    let deck = deck_with_prefix(
        &[
            diamond_four,
            club_five,
            QiGuiCard::suited(0, QiGuiSuit::Heart, QiGuiRank::Six),
            diamond_five,
        ],
        1,
    );
    let mut game = GameState::new_with_deck(rules, deck).unwrap();

    game.play_cards(QiGuiPlayerId(0), &[diamond_five]).unwrap();
    game.pass(QiGuiPlayerId(2)).unwrap();
    game.play_cards(QiGuiPlayerId(1), &[club_five]).unwrap();
    game.pass(QiGuiPlayerId(0)).unwrap();
    let outcome = game.pass(QiGuiPlayerId(2)).unwrap();

    assert_eq!(
        outcome,
        ActionOutcome::TrickCompleted {
            winner: QiGuiPlayerId(1),
            points: 10,
            next_player: QiGuiPlayerId(1),
            cards_drawn: 2,
        }
    );
    assert_eq!(game.player(QiGuiPlayerId(1)).unwrap().score(), 10);
    assert!(game.players().iter().all(|player| player.hand().len() == 5));
}

#[test]
fn emptying_a_hand_after_draw_pile_is_empty_collects_remaining_points() {
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let deck = build_deck(1);
    let mut game = GameState::new_with_deck(rules, deck).unwrap();
    let finisher = game.starting_card.player;
    let last_card = game.players[finisher.0].hand[0];

    // 构造规则边界状态：牌堆已耗尽，当前玩家只剩一张牌并领出。
    game.draw_pile.clear();
    game.players[finisher.0].hand = vec![last_card];
    game.trick = Some(TrickState::new(finisher));
    let hand_points_before: u32 = game
        .players
        .iter()
        .flat_map(|player| player.hand.iter())
        .map(|card| u32::from(card.score()))
        .sum();

    let outcome = game.play_cards(finisher, &[last_card]).unwrap();
    let ActionOutcome::GameFinished(result) = outcome else {
        panic!("game should finish");
    };
    assert_eq!(result.finisher, finisher);
    assert_eq!(
        result.captured_hand_points,
        hand_points_before - u32::from(last_card.score())
    );
}

#[test]
fn a_complete_deterministic_game_conserves_all_points() {
    let rules = QiGuiRuleSet {
        player_count: 3,
        ..QiGuiRuleSet::default()
    };
    let mut game = GameState::new_with_deck(rules, build_deck(1)).unwrap();

    for _ in 0..1_000 {
        if let Phase::Finished(result) = game.phase() {
            assert_eq!(result.scores.iter().sum::<u32>(), 100);
            return;
        }

        let trick = game.trick().unwrap();
        let current_player = trick.current_player();
        let current_play = trick.winning_play().cloned();
        let hand = game.player(current_player).unwrap().hand().to_vec();
        let playable = hand.into_iter().find(|card| {
            let candidate = classify(&[*card], game.rules()).unwrap();
            current_play
                .as_ref()
                .is_none_or(|current| can_beat(&candidate, current, game.rules()))
        });

        if let Some(card) = playable {
            game.play_cards(current_player, &[card]).unwrap();
        } else {
            game.pass(current_player).unwrap();
        }
    }

    panic!("deterministic game did not terminate");
}
