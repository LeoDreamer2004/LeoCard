use super::*;

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
    game.complete_bottom_flip().unwrap();

    let dealer = game.dealer().unwrap();
    let buried = game.players()[usize::from(dealer.0)].hand[..rules.kitty_size()].to_vec();
    game.bury(dealer, &buried).unwrap();

    assert_eq!(game.phase(), &Phase::Playing);
    assert!(game.bottom_copy().is_none());
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
