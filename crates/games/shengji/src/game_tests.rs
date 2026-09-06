use crate::play::classify_cards;
use crate::{ShengjiBidTrump, ShengjiSuit, ShengjiThrowPenalty};

use super::*;

fn dealt_game(rules: ShengjiRuleSet) -> GameState {
    let mut deck = build_deck_for(rules.deck_count);
    // 保证 0 号玩家在第一张就拿到方块 2，可以确定性亮主。
    let target = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let index = deck.iter().position(|card| *card == target).unwrap();
    deck.swap(0, index);
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_next().unwrap();
    game.declare(ShengjiPlayerId(0), &[target]).unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    game
}

#[test]
fn first_hand_final_counter_becomes_dealer_but_later_hands_keep_fixed_dealer() {
    let mut deck = build_deck();
    let diamond = [
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Two),
    ];
    let spade = [
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Two),
    ];
    for (target_index, card) in [
        (0, diamond[0]),
        (1, spade[0]),
        (4, diamond[1]),
        (5, spade[1]),
    ] {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let later_deck = deck.clone();
    let mut game = GameState::standard(deck).unwrap();
    for _ in 0..6 {
        game.deal_next().unwrap();
    }
    assert_eq!(game.dealer(), None);
    game.declare(ShengjiPlayerId(0), &diamond[..1]).unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(0)));
    game.declare(ShengjiPlayerId(1), &spade).unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(1)));
    assert_eq!(
        game.bidding().current().unwrap().trump,
        ShengjiBidTrump::Suit(ShengjiSuit::Spade)
    );
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(1)));

    let mut later = GameState::new(
        ShengjiRuleSet::default(),
        TeamProgress::default(),
        Some(ShengjiPlayerId(0)),
        ShengjiPlayerId(0),
        later_deck,
    )
    .unwrap();
    assert_eq!(later.dealer(), Some(ShengjiPlayerId(0)));
    for _ in 0..6 {
        later.deal_next().unwrap();
    }
    later.declare(ShengjiPlayerId(0), &diamond[..1]).unwrap();
    later.declare(ShengjiPlayerId(1), &spade).unwrap();
    assert_eq!(later.dealer(), Some(ShengjiPlayerId(0)));
    later.deal_all().unwrap();
    later.close_bidding_and_take_kitty().unwrap();
    assert_eq!(later.dealer(), Some(ShengjiPlayerId(0)));
    assert_eq!(later.trump().unwrap().suit, Some(ShengjiSuit::Spade));
}

#[test]
fn sequential_bottom_copies_change_trump_but_never_the_first_hands_dealer() {
    let rules = ShengjiRuleSet {
        // 配置开启不等于实际通过扳底定庄；正常亮主仍应允许抄底。
        bottom_flip: true,
        bottom_copy: true,
        ..ShengjiRuleSet::default()
    };
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let hearts = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
    ];
    let spades = [
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Two),
    ];
    let mut deck = build_deck();
    for (target_index, card) in [
        (0, diamond),
        (1, hearts[0]),
        (5, hearts[1]),
        (2, spades[0]),
        (6, spades[1]),
    ] {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    game.declare(ShengjiPlayerId(0), &[diamond]).unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    let first_bottom = game.players()[0].hand[..rules.kitty_size()].to_vec();
    game.bury(ShengjiPlayerId(0), &first_bottom).unwrap();
    assert_eq!(game.phase(), &Phase::BottomCopying);
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(1))
    );

    game.choose_bottom_copy(ShengjiPlayerId(1), Some(&hearts))
        .unwrap();
    assert_eq!(game.phase(), &Phase::BottomCopyBurying);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(0)));
    assert_eq!(game.trump().unwrap().suit, Some(ShengjiSuit::Heart));
    assert_eq!(game.players()[1].hand.len(), 33);
    let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
    game.bury(ShengjiPlayerId(1), &second_bottom).unwrap();
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(2))
    );

    game.choose_bottom_copy(ShengjiPlayerId(2), Some(&spades))
        .unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(0)));
    assert_eq!(game.trump().unwrap().suit, Some(ShengjiSuit::Spade));
    let final_bottom = game.players()[2].hand[..rules.kitty_size()].to_vec();
    game.bury(ShengjiPlayerId(2), &final_bottom).unwrap();
    while game.phase() == &Phase::BottomCopying {
        let player = game.bottom_copy().unwrap().current().unwrap();
        game.choose_bottom_copy(player, None).unwrap();
    }
    assert_eq!(game.phase(), &Phase::Playing);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(0)));
    assert_eq!(game.current_player(), Some(ShengjiPlayerId(0)));
}

#[test]
fn successful_bottom_copy_starts_a_new_round_and_non_dealer_original_bidder_can_copy_back() {
    let rules = ShengjiRuleSet {
        bottom_copy: true,
        ..ShengjiRuleSet::default()
    };
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let hearts = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
    ];
    let small_jokers = [ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
    let mut deck = build_deck();
    for (target_index, card) in [
        (0, diamond),
        (1, hearts[0]),
        (5, hearts[1]),
        (4, small_jokers[0]),
        (8, small_jokers[1]),
    ] {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        Some(ShengjiPlayerId(3)),
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    game.declare(ShengjiPlayerId(0), &[diamond]).unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    let first_bottom = game.players()[3]
        .hand
        .iter()
        .copied()
        .filter(|card| *card != diamond && !small_jokers.contains(card))
        .take(rules.kitty_size())
        .collect::<Vec<_>>();
    game.bury(ShengjiPlayerId(3), &first_bottom).unwrap();
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(1))
    );

    game.choose_bottom_copy(ShengjiPlayerId(1), Some(&hearts))
        .unwrap();
    let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
    game.bury(ShengjiPlayerId(1), &second_bottom).unwrap();

    // 2、3 号没有更强反主牌，询问会绕回最初亮主的 0 号。
    assert_eq!(game.phase(), &Phase::BottomCopying);
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(0))
    );
    game.choose_bottom_copy(ShengjiPlayerId(0), Some(&small_jokers))
        .unwrap();
    assert_eq!(game.phase(), &Phase::BottomCopyBurying);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(3)));
    assert_eq!(game.trump().unwrap().suit, None);
}

#[test]
fn dealer_cannot_copy_their_own_bottom_when_another_player_declared_trump() {
    let rules = ShengjiRuleSet {
        bottom_copy: true,
        ..ShengjiRuleSet::default()
    };
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let hearts = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
    ];
    let mut deck = build_deck();
    for (target_index, card) in [(1, diamond), (0, hearts[0]), (4, hearts[1])] {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        Some(ShengjiPlayerId(0)),
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    game.declare(ShengjiPlayerId(1), &[diamond]).unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(0)));

    let buried = game.players()[0]
        .hand
        .iter()
        .copied()
        .filter(|card| !hearts.contains(card))
        .take(rules.kitty_size())
        .collect::<Vec<_>>();
    game.bury(ShengjiPlayerId(0), &buried).unwrap();

    // 只有庄家持有能反方块单张的一对红桃级牌，但庄家不能抄自己的底。
    assert_eq!(game.phase(), &Phase::Playing);
    assert!(game.bottom_copy().is_none());
    assert_eq!(game.trump().unwrap().suit, Some(ShengjiSuit::Diamond));
}

#[test]
fn original_dealer_can_copy_after_another_player_reburies_the_bottom() {
    let rules = ShengjiRuleSet {
        bottom_copy: true,
        ..ShengjiRuleSet::default()
    };
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Two);
    let hearts = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Two),
    ];
    let small_jokers = [ShengjiCard::small_joker(0), ShengjiCard::small_joker(1)];
    let mut deck = build_deck();
    for (target_index, card) in [
        (0, diamond),
        (1, hearts[0]),
        (5, hearts[1]),
        (3, small_jokers[0]),
        (7, small_jokers[1]),
    ] {
        let index = deck
            .iter()
            .position(|candidate| *candidate == card)
            .unwrap();
        deck.swap(target_index, index);
    }
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        Some(ShengjiPlayerId(3)),
        ShengjiPlayerId(0),
        deck,
    )
    .unwrap();
    game.deal_all().unwrap();
    game.declare(ShengjiPlayerId(0), &[diamond]).unwrap();
    game.close_bidding_and_take_kitty().unwrap();

    let first_bottom = game.players()[3]
        .hand
        .iter()
        .copied()
        .filter(|card| !small_jokers.contains(card))
        .take(rules.kitty_size())
        .collect::<Vec<_>>();
    game.bury(ShengjiPlayerId(3), &first_bottom).unwrap();
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(1))
    );

    game.choose_bottom_copy(ShengjiPlayerId(1), Some(&hearts))
        .unwrap();
    let second_bottom = game.players()[1].hand[..rules.kitty_size()].to_vec();
    game.bury(ShengjiPlayerId(1), &second_bottom).unwrap();

    // 底牌已由 1 号重新埋过，最初的庄家 3 号不再是“上一位埋底者”。
    assert_eq!(
        game.bottom_copy().unwrap().current(),
        Some(ShengjiPlayerId(3))
    );
    game.choose_bottom_copy(ShengjiPlayerId(3), Some(&small_jokers))
        .unwrap();
    assert_eq!(game.phase(), &Phase::BottomCopyBurying);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(3)));
    assert_eq!(game.trump().unwrap().suit, None);
}

#[test]
fn bottom_flip_dealer_skips_bottom_copy_even_when_enabled() {
    let rules = ShengjiRuleSet {
        bottom_flip: true,
        bottom_copy: true,
        ..ShengjiRuleSet::default()
    };
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    game.flip_next_bottom_card().unwrap();
    let dealer = game.dealer().unwrap();
    game.complete_bottom_flip().unwrap();
    let buried = game.players()[usize::from(dealer.0)].hand[..rules.kitty_size()].to_vec();
    game.bury(dealer, &buried).unwrap();
    assert_eq!(game.phase(), &Phase::Playing);
    assert!(game.bottom_copy().is_none());
}

#[test]
fn nobody_declaring_requires_a_redeal() {
    let mut game = GameState::standard(build_deck()).unwrap();
    game.deal_all().unwrap();
    assert_eq!(
        game.close_bidding_and_take_kitty(),
        Err(GameError::RedealRequired)
    );
    assert_eq!(game.phase(), &Phase::RedealRequired);
}

#[test]
fn power_outage_keeps_hands_rotates_dealer_and_uses_the_new_teams_level() {
    let rules = ShengjiRuleSet {
        power_outage_dealer: true,
        ..ShengjiRuleSet::default()
    };
    let teams = TeamProgress {
        levels: [ShengjiRank::Ten, ShengjiRank::Five],
    };
    let mut game = GameState::new(
        rules,
        teams,
        Some(ShengjiPlayerId(0)),
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    game.deal_all().unwrap();
    let hands = game
        .players()
        .iter()
        .map(|player| player.hand.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        game.close_bidding_and_take_kitty().unwrap(),
        ActionOutcome::PowerOutageDealerChanged {
            dealer: ShengjiPlayerId(1)
        }
    );
    assert_eq!(game.phase(), &Phase::BiddingGrace);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(1)));
    assert_eq!(game.bidding().level(), ShengjiRank::Five);
    assert_eq!(
        game.players()
            .iter()
            .map(|player| player.hand.clone())
            .collect::<Vec<_>>(),
        hands
    );

    let bidder = ShengjiPlayerId(3);
    let bid = game.players()[usize::from(bidder.0)]
        .hand
        .iter()
        .copied()
        .find(|card| card.rank() == ShengjiRank::Five)
        .unwrap();
    game.declare(bidder, &[bid]).unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(1)));
    assert_eq!(game.trump().unwrap().level, ShengjiRank::Five);
    assert_eq!(game.trump().unwrap().suit, bid.suit());
}

#[test]
fn a_second_power_outage_redeals_unless_bottom_flip_is_enabled() {
    let rules = ShengjiRuleSet {
        power_outage_dealer: true,
        ..ShengjiRuleSet::default()
    };
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        Some(ShengjiPlayerId(0)),
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    game.deal_all().unwrap();
    game.close_bidding_and_take_kitty().unwrap();
    assert_eq!(
        game.close_bidding_and_take_kitty(),
        Err(GameError::RedealRequired)
    );
}

#[test]
fn bottom_flip_reveals_matches_and_sets_dealer_level_and_suit() {
    let rules = ShengjiRuleSet {
        bottom_flip: true,
        ..ShengjiRuleSet::default()
    };
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    game.deal_all().unwrap();
    assert_eq!(
        game.close_bidding_and_take_kitty().unwrap(),
        ActionOutcome::BottomFlipStarted
    );
    let ActionOutcome::BottomCardRevealed(reveal) = game.flip_next_bottom_card().unwrap() else {
        unreachable!();
    };
    assert_eq!(
        reveal.card,
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Nine)
    );
    assert_eq!(reveal.dealer, Some(ShengjiPlayerId(2)));
    assert_eq!(reveal.matches.len(), 1);
    assert_eq!(reveal.matches[0].player, ShengjiPlayerId(2));
    assert_eq!(game.phase(), &Phase::BottomFlipping);
    assert_eq!(game.dealer(), Some(ShengjiPlayerId(2)));
    assert_eq!(game.trump().unwrap().level, ShengjiRank::Two);
    assert_eq!(game.trump().unwrap().suit, Some(ShengjiSuit::Spade));
    assert_eq!(game.players()[2].hand.len(), 25);
    game.complete_bottom_flip().unwrap();
    assert_eq!(game.phase(), &Phase::Burying);
    assert_eq!(game.players()[2].hand.len(), 33);
}

#[test]
fn bottom_flip_dealer_selection_obeys_team_total_personal_count_and_distance() {
    let cards = |count: usize| {
        (0..count)
            .map(|deck| ShengjiCard::suited(deck as u8, ShengjiSuit::Heart, ShengjiRank::Ace))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        select_bottom_flip_dealer(
            &[BottomFlipMatch {
                player: ShengjiPlayerId(3),
                cards: cards(1),
            }],
            ShengjiPlayerId(0),
        ),
        Some(ShengjiPlayerId(3))
    );
    assert_eq!(
        select_bottom_flip_dealer(
            &[
                BottomFlipMatch {
                    player: ShengjiPlayerId(0),
                    cards: cards(1),
                },
                BottomFlipMatch {
                    player: ShengjiPlayerId(1),
                    cards: cards(1),
                },
            ],
            ShengjiPlayerId(0),
        ),
        None
    );
    assert_eq!(
        select_bottom_flip_dealer(
            &[
                BottomFlipMatch {
                    player: ShengjiPlayerId(0),
                    cards: cards(2),
                },
                BottomFlipMatch {
                    player: ShengjiPlayerId(2),
                    cards: cards(1),
                },
                BottomFlipMatch {
                    player: ShengjiPlayerId(1),
                    cards: cards(2),
                },
            ],
            ShengjiPlayerId(1),
        ),
        Some(ShengjiPlayerId(0))
    );
    assert_eq!(
        select_bottom_flip_dealer(
            &[
                BottomFlipMatch {
                    player: ShengjiPlayerId(0),
                    cards: cards(2),
                },
                BottomFlipMatch {
                    player: ShengjiPlayerId(2),
                    cards: cards(2),
                },
                BottomFlipMatch {
                    player: ShengjiPlayerId(1),
                    cards: cards(3),
                },
            ],
            ShengjiPlayerId(1),
        ),
        Some(ShengjiPlayerId(2))
    );
}

#[test]
fn dealer_may_bury_any_eight_cards() {
    let mut game = dealt_game(ShengjiRuleSet::default());
    let dealer = game.dealer().unwrap();
    let buried = game.players()[usize::from(dealer.0)].hand[..8].to_vec();
    game.bury(dealer, &buried).unwrap();
    assert_eq!(game.buried(), buried);
    assert_eq!(game.bottom_burier(), Some(dealer));
    assert_eq!(game.players()[usize::from(dealer.0)].hand.len(), 25);
    assert_eq!(game.current_player(), Some(dealer));
}

#[test]
fn constant_trump_first_hand_starts_both_teams_at_three() {
    let rules = ShengjiRuleSet {
        constant_trump: true,
        ..ShengjiRuleSet::default()
    };
    let game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();

    assert_eq!(
        game.teams().levels,
        [ShengjiRank::Three, ShengjiRank::Three]
    );
    assert_eq!(game.bidding().level(), ShengjiRank::Three);
}

#[test]
fn dealer_throw_penalty_adds_to_collectors_and_collectors_never_go_below_zero() {
    let mut game = dealt_game(ShengjiRuleSet {
        throw_penalty: ShengjiThrowPenalty::TenPerCard,
        ..ShengjiRuleSet::default()
    });
    game.dealer = Some(ShengjiPlayerId(0));
    game.apply_throw_penalty(ShengjiTeamId(0), 50);
    assert_eq!(game.collecting_score(), 50);
    game.apply_throw_penalty(ShengjiTeamId(1), 80);
    assert_eq!(game.collecting_score(), 0);
    game.apply_throw_penalty(ShengjiTeamId(1), 10);
    assert_eq!(game.collecting_score(), 0);
}

#[test]
fn failed_throw_keeps_the_attempt_visible_but_only_plays_the_forced_low_card() {
    let mut game = GameState::standard(build_deck()).unwrap();
    let low = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Three);
    let high = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Nine);
    let beating_card = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Four);
    game.rules.throw_penalty = ShengjiThrowPenalty::TenPerCard;
    game.phase = Phase::Playing;
    game.trump = Some(ShengjiTrump::new(ShengjiRank::Two, Some(ShengjiSuit::Spade)).unwrap());
    game.dealer = Some(ShengjiPlayerId(0));
    game.current_player = Some(ShengjiPlayerId(0));
    game.players[0].hand = vec![low, high];
    game.players[1].hand = vec![beating_card];

    let attempted = vec![high, low];
    let outcome = game.play_cards(ShengjiPlayerId(0), &attempted).unwrap();
    let ActionOutcome::ThrowFailed {
        player,
        attempted: visible_attempt,
        forced,
        penalty_points,
        next,
    } = outcome
    else {
        panic!("the beatable mixed single throw should fail");
    };

    assert_eq!(player, ShengjiPlayerId(0));
    assert_eq!(visible_attempt, attempted);
    assert_eq!(forced.cards, vec![low]);
    assert_eq!(penalty_points, 20);
    assert_eq!(next, ShengjiPlayerId(1));
    assert_eq!(game.players[0].hand, vec![high]);
    assert_eq!(game.collecting_score(), 20);
    assert_eq!(game.current_player(), Some(ShengjiPlayerId(1)));
}

#[test]
fn settlement_thresholds_and_next_dealers_match_the_declared_rules() {
    let cases = [
        (0, ShengjiTeamId(0), 3, ShengjiPlayerId(2)),
        (35, ShengjiTeamId(0), 2, ShengjiPlayerId(2)),
        (75, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (80, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (120, ShengjiTeamId(1), 1, ShengjiPlayerId(1)),
        (160, ShengjiTeamId(1), 2, ShengjiPlayerId(1)),
        (200, ShengjiTeamId(1), 3, ShengjiPlayerId(1)),
    ];
    for (score, promoted_team, steps, next_dealer) in cases {
        let mut game = dealt_game(ShengjiRuleSet::default());
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team);
        assert_eq!(result.promoted_steps, steps);
        assert_eq!(result.next_dealer, next_dealer);
    }
}

#[test]
fn three_deck_settlement_uses_sixty_point_bands() {
    let cases = [
        (0, ShengjiTeamId(0), 3, ShengjiPlayerId(2)),
        (59, ShengjiTeamId(0), 2, ShengjiPlayerId(2)),
        (60, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (119, ShengjiTeamId(0), 1, ShengjiPlayerId(2)),
        (120, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (179, ShengjiTeamId(1), 0, ShengjiPlayerId(1)),
        (180, ShengjiTeamId(1), 1, ShengjiPlayerId(1)),
        (240, ShengjiTeamId(1), 2, ShengjiPlayerId(1)),
    ];
    for (score, promoted_team, steps, next_dealer) in cases {
        let mut game = dealt_game(ShengjiRuleSet {
            deck_count: 3,
            ..ShengjiRuleSet::default()
        });
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team, "score={score}");
        assert_eq!(result.promoted_steps, steps, "score={score}");
        assert_eq!(result.next_dealer, next_dealer, "score={score}");
    }
}

#[test]
fn four_deck_settlement_uses_eighty_point_bands() {
    let cases = [
        (0, ShengjiTeamId(0), 3),
        (79, ShengjiTeamId(0), 2),
        (80, ShengjiTeamId(0), 1),
        (159, ShengjiTeamId(0), 1),
        (160, ShengjiTeamId(1), 0),
        (239, ShengjiTeamId(1), 0),
        (240, ShengjiTeamId(1), 1),
        (320, ShengjiTeamId(1), 2),
    ];
    for (score, promoted_team, steps) in cases {
        let mut game = dealt_game(ShengjiRuleSet {
            deck_count: 4,
            ..ShengjiRuleSet::default()
        });
        game.dealer = Some(ShengjiPlayerId(0));
        game.collecting_score = score;
        game.buried.clear();
        let play = classify_cards(
            &[ShengjiCard::suited(
                0,
                ShengjiSuit::Diamond,
                ShengjiRank::Three,
            )],
            game.trump.unwrap(),
        )
        .unwrap();
        let last = TrickRecord {
            leader: ShengjiPlayerId(0),
            plays: vec![(ShengjiPlayerId(0), play)],
            winner: ShengjiPlayerId(0),
            points: 0,
        };
        let result = game.finish_hand(ShengjiPlayerId(0), &last);
        assert_eq!(result.promoted_team, promoted_team, "score={score}");
        assert_eq!(result.promoted_steps, steps, "score={score}");
    }
}

fn five_trump_crossing_game(trump_suit: Option<ShengjiSuit>) -> (GameState, Vec<ShengjiCard>) {
    let rules = ShengjiRuleSet {
        five_trump_crossing: true,
        ..ShengjiRuleSet::default()
    };
    let mut game = GameState::new(
        rules,
        TeamProgress::default(),
        None,
        ShengjiPlayerId(0),
        build_deck(),
    )
    .unwrap();
    let buried = [
        ShengjiRank::Three,
        ShengjiRank::Four,
        ShengjiRank::Five,
        ShengjiRank::Six,
        ShengjiRank::Seven,
        ShengjiRank::Eight,
        ShengjiRank::Nine,
        ShengjiRank::Jack,
    ]
    .into_iter()
    .map(|rank| ShengjiCard::suited(0, ShengjiSuit::Diamond, rank))
    .collect::<Vec<_>>();
    let player_zero = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::big_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Six),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Seven),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Eight),
    ];
    let player_two = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Five),
        ShengjiCard::small_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Five),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Six),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Seven),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Eight),
    ];
    game.players[0].hand = [buried.as_slice(), player_zero.as_slice()].concat();
    game.players[1].hand = [
        ShengjiRank::Six,
        ShengjiRank::Seven,
        ShengjiRank::Eight,
        ShengjiRank::Nine,
        ShengjiRank::Jack,
        ShengjiRank::Queen,
    ]
    .into_iter()
    .map(|rank| ShengjiCard::suited(0, ShengjiSuit::Heart, rank))
    .collect();
    game.players[2].hand = player_two.to_vec();
    game.players[3].hand = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::King),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ace),
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten),
        ShengjiCard::big_joker(1),
    ];
    game.trump = Some(ShengjiTrump::new(ShengjiRank::Ten, trump_suit).unwrap());
    game.dealer = Some(ShengjiPlayerId(0));
    game.phase = Phase::Burying;
    (game, buried)
}

#[test]
fn teammates_can_cross_each_other_once_with_both_transfers_applied_simultaneously() {
    let (mut game, buried) = five_trump_crossing_game(Some(ShengjiSuit::Heart));
    game.bury(ShengjiPlayerId(0), &buried).unwrap();
    assert_eq!(game.phase(), &Phase::FiveTrumpCrossing);
    let state = game.five_trump_crossing().unwrap();
    assert!(state.eligible(ShengjiPlayerId(0)));
    assert!(state.eligible(ShengjiPlayerId(2)));
    assert!(!state.eligible(ShengjiPlayerId(1)));
    assert!(!state.eligible(ShengjiPlayerId(3)));
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(1), None),
        Err(GameError::CrossingNotEligible)
    );

    let initial_zero = game.players[0].hand.clone();
    let initial_two = game.players[2].hand.clone();
    let zero_out = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::big_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
    ];
    let invalid_zero = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Five),
    ];
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(0), Some(&invalid_zero)),
        Err(GameError::CrossingMustIncludeAllTrumps)
    );
    game.choose_five_trump_crossing(ShengjiPlayerId(0), Some(&zero_out))
        .unwrap();
    assert_eq!(game.players[0].hand, initial_zero, "决定阶段不能提前移动牌");

    let two_out = vec![
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Five),
        ShengjiCard::small_joker(0),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Three),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Four),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Five),
    ];
    game.choose_five_trump_crossing(ShengjiPlayerId(2), Some(&two_out))
        .unwrap();
    assert_eq!(
        game.five_trump_crossing().unwrap().stage(),
        FiveTrumpCrossingStage::Returning
    );
    assert_eq!(game.players[0].hand.len(), initial_zero.len());
    assert_eq!(game.players[2].hand.len(), initial_two.len());

    let after_outgoing_zero = game.players[0].hand.clone();
    game.return_five_trump_crossing(ShengjiPlayerId(0), &two_out)
        .unwrap();
    assert_eq!(
        game.players[0].hand, after_outgoing_zero,
        "回牌也必须等所有人选定后再同时移动"
    );
    game.return_five_trump_crossing(ShengjiPlayerId(2), &zero_out)
        .unwrap();
    assert_eq!(game.phase(), &Phase::Playing);

    let mut final_zero = game.players[0].hand.clone();
    let mut final_two = game.players[2].hand.clone();
    let mut initial_zero = initial_zero;
    let mut initial_two = initial_two;
    for hand in [
        &mut final_zero,
        &mut final_two,
        &mut initial_zero,
        &mut initial_two,
    ] {
        hand.sort_by(ShengjiCard::identity_cmp);
    }
    assert_eq!(final_zero, initial_zero);
    assert_eq!(final_two, initial_two);
    assert_eq!(
        game.choose_five_trump_crossing(ShengjiPlayerId(0), None),
        Err(GameError::WrongPhase),
        "一局只能进行一次五主过江"
    );
}

#[test]
fn five_trump_crossing_is_skipped_in_no_trump_and_all_declines_start_play() {
    let (mut no_trump, buried) = five_trump_crossing_game(None);
    no_trump.bury(ShengjiPlayerId(0), &buried).unwrap();
    assert_eq!(no_trump.phase(), &Phase::Playing);
    assert!(no_trump.five_trump_crossing().is_none());

    let (mut suited, buried) = five_trump_crossing_game(Some(ShengjiSuit::Heart));
    suited.bury(ShengjiPlayerId(0), &buried).unwrap();
    suited
        .choose_five_trump_crossing(ShengjiPlayerId(0), None)
        .unwrap();
    suited
        .choose_five_trump_crossing(ShengjiPlayerId(2), None)
        .unwrap();
    assert_eq!(suited.phase(), &Phase::Playing);
    assert_eq!(suited.current_player(), suited.dealer());
}
