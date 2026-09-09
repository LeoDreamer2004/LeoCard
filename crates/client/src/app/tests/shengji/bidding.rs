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
fn previous_trick_button_appears_only_after_playing_starts() {
    assert_eq!(
        shengji_previous_trick_button_state(
            &ShengjiPhaseView::Dealing {
                cards_remaining: 80,
            },
            false,
        ),
        None
    );
    assert_eq!(
        shengji_previous_trick_button_state(&ShengjiPhaseView::Burying, false),
        None
    );
    assert_eq!(
        shengji_previous_trick_button_state(&ShengjiPhaseView::Playing, false),
        Some(false)
    );
    assert_eq!(
        shengji_previous_trick_button_state(&ShengjiPhaseView::Playing, true),
        Some(true)
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
fn joker_bidding_button_reuses_the_current_joker_for_protection_and_no_trump() {
    let heart = [
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let big = [ShengjiCard::big_joker(0), ShengjiCard::big_joker(1)];
    let mut game = shengji_ui_snapshot(
        [heart.as_slice(), big.as_slice()].concat(),
        Some(ShengjiDeclarationView {
            player: PlayerId(0),
            trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
            kind: ShengjiBidKind::Initial,
            protected: false,
            cards: vec![heart[0], big[0]],
        }),
    );
    game.rules.bid_with_joker = true;
    game.your_exposed_cards = vec![heart[0], big[0]];

    // 同花色加亮时当前展示的大王继续使用，只需提交新增的级牌。
    assert_eq!(
        shengji_declaration_candidate(&game, Some(ShengjiSuit::Heart)),
        Some(vec![heart[1]])
    );
    // 反无主时，已经展示的大王可以与手里的另一张大王组成一对。
    assert_eq!(
        shengji_declaration_candidate(&game, None),
        Some(big.to_vec())
    );
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

#[test]
fn four_deck_hand_reveal_keeps_all_fifty_two_cards_inside_design_width() {
    let reveal = shengji_hand_card_reveal(52);
    let width = reveal * 51.0 + CardSize::Hand.dimensions().0;
    assert!(reveal < HAND_CARD_REVEAL);
    assert!(width <= DESIGN_WIDTH - 96.0 + f32::EPSILON);
}
