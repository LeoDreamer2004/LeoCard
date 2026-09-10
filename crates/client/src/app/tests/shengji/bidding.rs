use super::prelude::*;

#[test]
fn shengji_bidding_buttons_choose_single_protection_pairs_and_no_trump() {
    let diamond = [
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Ten),
    ];
    let heart = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let hand = [diamond.as_slice(), heart.as_slice(), big.as_slice()].concat();
    let mut game = shengji_ui_snapshot(hand, None);

    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Diamond)),
        Some(vec![diamond[0]])
    );
    game.declaration = Some(ShengjiDeclarationView {
        player: game.you,
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
        kind: ShengjiBidKind::Initial,
        protected: false,
        cards: vec![diamond[0]],
    });
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Diamond)),
        Some(vec![diamond[1]])
    );
    game.declaration.as_mut().unwrap().player = PlayerId(1);
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(heart.to_vec())
    );
    assert_eq!(
        shengji_declaration_candidate(&game, None),
        Some(big.to_vec())
    );
}

#[test]
fn next_hand_bidding_uses_the_authoritative_level_with_the_public_dealer() {
    let three = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Three);
    let two = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two);
    let mut game = shengji_ui_snapshot(vec![two, three], None);
    // 模拟上一局换庄：0 队仍打 2，下一庄所在的 1 队已经打 3。发牌阶段
    // 已直接公开下一庄，抢亮仍以服务端明确给出的 bidding_level 为准。
    game.levels = [ShengjiRank::Two, ShengjiRank::Three];
    game.bidding_level = ShengjiRank::Three;
    game.dealer = Some(PlayerId(1));

    assert_eq!(shengji_current_level(&game), ShengjiRank::Three);
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(vec![three])
    );

    game.declaration = Some(ShengjiDeclarationView {
        player: game.you,
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
        kind: ShengjiBidKind::Initial,
        protected: false,
        cards: vec![three],
    });
    assert_eq!(
        shengji_display_trump(&game).map(|trump| trump.level),
        Some(ShengjiRank::Three)
    );
}

#[test]
fn joker_bidding_buttons_require_the_matching_joker_and_hide_initial_no_trump() {
    let heart = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let big = ShengjiCard::big_joker(0);
    let small = ShengjiCard::small_joker(0);
    let mut game = shengji_ui_snapshot(vec![heart, big, small], None);
    game.rules.bid_with_joker = true;

    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(vec![heart, big])
    );
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Spade)),
        None
    );
    assert_eq!(shengji_declaration_candidate(&game, None), None);
}

#[test]
fn three_deck_joker_bidding_strength_ignores_the_companion_joker_count() {
    let heart = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let own_big = ShengjiCard::big_joker(0);
    let mut game = shengji_ui_snapshot(
        [heart.as_slice(), &[own_big]].concat(),
        Some(ShengjiDeclarationView {
            player: PlayerId(1),
            trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
            kind: ShengjiBidKind::Initial,
            protected: false,
            cards: vec![
                ShengjiCard::suited(2, ShengjiSuit::Diamond, ShengjiRank::Ten),
                ShengjiCard::big_joker(2),
            ],
        }),
    );
    game.rules.deck_count = 3;
    game.rules.bid_with_joker = true;

    // 当前声明虽展示两张牌，强度仍只是一张级牌；反主应使用两张级牌加王。
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(vec![heart[0], heart[1], own_big])
    );
}

#[test]
fn three_deck_bidding_buttons_skip_single_counters_and_choose_the_lowest_stronger_level() {
    let diamond = [
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(2, ShengjiSuit::Diamond, ShengjiRank::Ten),
    ];
    let heart = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(2, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let small = [
        ShengjiCard::small_joker(0),
        ShengjiCard::small_joker(1),
        ShengjiCard::small_joker(2),
    ];
    let mut game = shengji_ui_snapshot(
        [diamond.as_slice(), heart.as_slice(), small.as_slice()].concat(),
        None,
    );
    game.rules.deck_count = 3;
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(vec![heart[0]])
    );

    game.declaration = Some(ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
        kind: ShengjiBidKind::Initial,
        protected: false,
        cards: vec![diamond[0]],
    });
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(heart[..2].to_vec())
    );
    assert_eq!(
        shengji_declaration_candidate(&game, None),
        Some(small[..2].to_vec())
    );

    game.declaration = Some(ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::NoTrumpSmallJoker,
        kind: ShengjiBidKind::Counter,
        protected: false,
        cards: small[..2].to_vec(),
    });
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(heart.to_vec())
    );
}

#[test]
fn four_deck_bidding_button_reaches_quad_level() {
    let hearts = (0..4)
        .map(|deck| ShengjiCard::suited(deck, ShengjiSuit::Heart, ShengjiRank::Ten))
        .collect::<Vec<_>>();
    let mut game = shengji_ui_snapshot(
        hearts.clone(),
        Some(ShengjiDeclarationView {
            player: PlayerId(1),
            trump: ShengjiBidTrump::NoTrumpBigJoker,
            kind: ShengjiBidKind::Counter,
            protected: false,
            cards: (0..3).map(ShengjiCard::big_joker).collect(),
        }),
    );
    game.rules.deck_count = 4;
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(hearts)
    );
}
