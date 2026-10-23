use super::prelude::*;
use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerId, ProfileId, SeatId, ShengjiDeclarationView,
    ShengjiPhaseView, ShengjiPlayerState, ShengjiPublicPlay, ShengjiSnapshot, ShengjiTrickView,
};
use leocard_shengji::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrump, TrickPlay, classify_lead,
};

fn shengji_ui_snapshot(
    hand: Vec<ShengjiCard>,
    declaration: Option<ShengjiDeclarationView>,
) -> ShengjiSnapshot {
    ShengjiSnapshot {
        match_id: MatchId([7; 16]),
        hand_number: 1,
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: ShengjiRuleSet::default(),
        players: (0..4)
            .map(|id| ShengjiPlayerState {
                id: PlayerId(id),
                profile_id: ProfileId([id; 32]),
                name: format!("玩家{id}"),
                avatar: None,
                seat: SeatId(id),
                missing_suits: Vec::new(),
                hand_len: if id == 0 { hand.len() as u8 } else { 0 },
                ready: false,
                connected: true,
                auto_play: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            })
            .collect(),
        your_hand: hand,
        your_exposed_cards: declaration
            .as_ref()
            .filter(|declaration| declaration.player == PlayerId(0))
            .map_or_else(Vec::new, |declaration| declaration.cards.clone()),
        levels: [ShengjiRank::Ten, ShengjiRank::Ten],
        bidding_level: ShengjiRank::Ten,
        dealer: None,
        trump: None,
        declaration,
        bottom_copy_decisions: Vec::new(),
        current_player: None,
        trick: None,
        played_cards: Vec::new(),
        throw_failure: None,
        collecting_score: 0,
        buried_count: 0,
        your_buried: Vec::new(),
        phase: ShengjiPhaseView::Dealing {
            cards_remaining: 80,
        },
    }
}

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

#[test]
fn shengji_turn_sync_preselects_the_only_required_pair() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let pair = |rank| {
        [
            ShengjiCard::suited(0, ShengjiSuit::Spade, rank),
            ShengjiCard::suited(1, ShengjiSuit::Spade, rank),
        ]
    };
    let threes = pair(ShengjiRank::Three);
    let hand = [
        threes.as_slice(),
        &[
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Six),
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Seven),
            ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Nine),
        ],
    ]
    .concat();
    let lead_cards = [
        pair(ShengjiRank::Jack).as_slice(),
        pair(ShengjiRank::Queen).as_slice(),
    ]
    .concat();
    let TrickPlay::Accepted(lead) =
        classify_lead(&lead_cards, trump, &ShengjiRuleSet::default(), &[]).unwrap()
    else {
        unreachable!("a single tractor is not a throw")
    };
    let mut game = shengji_ui_snapshot(hand, None);
    game.phase = ShengjiPhaseView::Playing;
    game.trump = Some(trump);
    game.current_player = Some(game.you);
    game.trick = Some(ShengjiTrickView {
        leader: PlayerId(1),
        current_player: game.you,
        winning_player: PlayerId(1),
        plays: vec![ShengjiPublicPlay {
            player: PlayerId(1),
            play: lead,
            throw_penalty: 0,
        }],
        table_points: 0,
    });
    let mut ui = ShengjiUiState::default();

    select_forced_shengji_follow_cards(&game, &mut ui);

    assert_eq!(ui.selected, threes.into_iter().collect());

    let first = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = first.iter().copied().collect();
    let second = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = second.iter().copied().collect();
    let third = next_shengji_hint(&game, &ui.selected).unwrap();
    ui.selected = third.iter().copied().collect();
    let wrapped = next_shengji_hint(&game, &ui.selected).unwrap();

    assert_ne!(first, second);
    assert_ne!(second, third);
    assert_eq!(wrapped, first);
}

#[test]
fn shengji_hand_sort_keeps_all_trumps_before_side_suits() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let big = ShengjiCard::big_joker(0);
    let main_level = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let off_level = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let side_ace = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let mut cards = vec![side_ace, off_level, main_level, big];

    sort_shengji_cards(&mut cards, Some(trump));

    assert_eq!(cards, vec![big, main_level, off_level, side_ace]);
}

#[test]
fn shengji_hand_sort_places_unbid_level_cards_immediately_after_jokers() {
    let big = ShengjiCard::big_joker(0);
    let small = ShengjiCard::small_joker(0);
    let spade = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let heart = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let club = ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten);
    let diamond = ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten);
    let side_ace = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let game = shengji_ui_snapshot(Vec::new(), None);
    let mut cards = vec![side_ace, diamond, small, club, big, heart, spade];

    assert_eq!(shengji_display_trump(&game), None);
    sort_shengji_cards(&mut cards, shengji_hand_sort_trump(&game));

    assert_eq!(
        cards,
        vec![big, small, spade, heart, club, diamond, side_ace]
    );
}

#[test]
fn shengji_hand_sort_groups_off_suit_level_pairs_in_spade_heart_club_diamond_order() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, None).unwrap();
    let spades = [
        ShengjiCard::suited(1, ShengjiSuit::Spade, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten),
    ];
    let hearts = [
        ShengjiCard::suited(1, ShengjiSuit::Heart, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten),
    ];
    let clubs = [
        ShengjiCard::suited(1, ShengjiSuit::Club, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Club, ShengjiRank::Ten),
    ];
    let diamonds = [
        ShengjiCard::suited(1, ShengjiSuit::Diamond, ShengjiRank::Ten),
        ShengjiCard::suited(0, ShengjiSuit::Diamond, ShengjiRank::Ten),
    ];
    let mut cards = vec![
        diamonds[0],
        spades[1],
        hearts[0],
        clubs[1],
        spades[0],
        diamonds[1],
        clubs[0],
        hearts[1],
    ];

    sort_shengji_cards(&mut cards, Some(trump));

    assert_eq!(
        cards,
        [spades, hearts, clubs, diamonds]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    );
}
