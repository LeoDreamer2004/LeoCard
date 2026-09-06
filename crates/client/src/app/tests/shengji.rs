use super::*;
use bevy::ui::RelativeCursorPosition;

use leocard_protocol::{
    GamePhaseView, MatchId, PlayerGameProfiles, PlayerId, PlayerScore, ProfileId, QiGui523Snapshot,
    SeatId, ShengjiDeclarationView, ShengjiPhaseView, ShengjiPlayerState, ShengjiPublicPlay,
    ShengjiSnapshot, ShengjiTrickView, StartingCardView, TurnTimerView,
};
use leocard_qigui523::{QiGuiCard, QiGuiRank, QiGuiSuit};

use leocard_shengji::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrump, TrickPlay, classify_lead,
};

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
    let mut ui = UiState::default();

    select_forced_shengji_follow_cards(&game, &mut ui);

    assert_eq!(ui.shengji.selected, threes.into_iter().collect());

    let first = next_shengji_hint(&game, &ui.shengji.selected).unwrap();
    ui.shengji.selected = first.iter().copied().collect();
    let second = next_shengji_hint(&game, &ui.shengji.selected).unwrap();
    ui.shengji.selected = second.iter().copied().collect();
    let third = next_shengji_hint(&game, &ui.shengji.selected).unwrap();
    ui.shengji.selected = third.iter().copied().collect();
    let wrapped = next_shengji_hint(&game, &ui.shengji.selected).unwrap();

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
fn shengji_constant_trump_sort_places_main_and_off_twos_below_level_cards() {
    let trump = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart))
        .unwrap()
        .with_constant_trump(true);
    let main_level = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let off_level = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let main_two = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Two);
    let off_two = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Two);
    let trump_ace = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ace);
    let mut cards = vec![off_two, trump_ace, main_level, main_two, off_level];

    sort_shengji_cards(&mut cards, Some(trump));

    assert_eq!(
        cards,
        vec![main_level, off_level, main_two, off_two, trump_ace]
    );
    assert_eq!(shengji_trump_star_count(main_two, Some(trump)), 1);
    assert_eq!(shengji_trump_star_count(off_two, Some(trump)), 1);
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

#[test]
fn shengji_trump_stars_distinguish_main_level_and_other_trumps() {
    let suited = ShengjiTrump::new(ShengjiRank::Ten, Some(ShengjiSuit::Heart)).unwrap();
    let main_level = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten);
    let off_level = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ten);
    let suit_card = ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Nine);
    let side_card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let joker = ShengjiCard::big_joker(0);

    assert_eq!(shengji_trump_star_count(main_level, Some(suited)), 2);
    assert_eq!(shengji_trump_star_count(joker, Some(suited)), 2);
    assert_eq!(shengji_trump_star_count(off_level, Some(suited)), 1);
    assert_eq!(shengji_trump_star_count(suit_card, Some(suited)), 1);
    assert_eq!(shengji_trump_star_count(side_card, Some(suited)), 0);

    let no_trump = ShengjiTrump::new(ShengjiRank::Ten, None).unwrap();
    assert_eq!(shengji_trump_star_count(off_level, Some(no_trump)), 1);
}

#[test]
fn timer_only_snapshots_are_eligible_for_in_place_ui_updates() {
    let mut before = QiGui523Snapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: Vec::new(),
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: StartingCardView {
            player: PlayerId(0),
            card: QiGuiCard::suited(0, QiGuiSuit::Diamond, QiGuiRank::Four),
        },
        trick: None,
        turn_timer: Some(TurnTimerView {
            player: PlayerId(0),
            base_seconds: 5,
            reserve_seconds: 30,
        }),
        phase: GamePhaseView::Playing,
    };
    let mut after = before.clone();
    after.turn_timer.as_mut().unwrap().base_seconds = 4;

    assert!(only_turn_timer_changed(Some(&before), Some(&after)));

    before.draw_pile_len = 1;
    assert!(!only_turn_timer_changed(Some(&before), Some(&after)));
}

#[test]
fn shengji_remote_deals_and_hidden_grace_ticks_skip_full_ui_rebuilds() {
    let own_card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let mut before = shengji_ui_snapshot(vec![own_card], None);
    before.players[0].hand_len = 1;
    let mut remote_deal = before.clone();
    remote_deal.phase = ShengjiPhaseView::Dealing {
        cards_remaining: 79,
    };
    remote_deal.players[1].hand_len = 2;
    assert!(only_shengji_transient_progress_changed(
        Some(&before),
        Some(&remote_deal)
    ));

    let mut own_deal = remote_deal.clone();
    own_deal.your_hand.push(ShengjiCard::suited(
        0,
        ShengjiSuit::Heart,
        ShengjiRank::King,
    ));
    own_deal.players[0].hand_len = 2;
    assert!(!only_shengji_transient_progress_changed(
        Some(&remote_deal),
        Some(&own_deal)
    ));

    let mut grace_before = remote_deal;
    grace_before.phase = ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining: 5_000,
        power_outage: false,
        confirmed_count: 0,
        you_confirmed: false,
    };
    let mut grace_after = grace_before.clone();
    grace_after.phase = ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining: 4_900,
        power_outage: false,
        confirmed_count: 0,
        you_confirmed: false,
    };
    assert!(only_shengji_transient_progress_changed(
        Some(&grace_before),
        Some(&grace_after)
    ));

    grace_after.phase = ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining: 4_900,
        power_outage: false,
        confirmed_count: 1,
        you_confirmed: true,
    };
    assert!(!only_shengji_transient_progress_changed(
        Some(&grace_before),
        Some(&grace_after)
    ));

    grace_after.phase = ShengjiPhaseView::BiddingGrace {
        milliseconds_remaining: 4_900,
        power_outage: false,
        confirmed_count: 0,
        you_confirmed: false,
    };

    grace_after.declaration = Some(ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
        kind: ShengjiBidKind::Initial,
        protected: false,
        cards: vec![ShengjiCard::suited(0, ShengjiSuit::Heart, ShengjiRank::Ten)],
    });
    assert!(!only_shengji_transient_progress_changed(
        Some(&grace_before),
        Some(&grace_after)
    ));

    let mut copy_before = grace_before.clone();
    copy_before.phase = ShengjiPhaseView::BottomCopying {
        player: PlayerId(1),
        milliseconds_remaining: 10_000,
    };
    let mut copy_after = copy_before.clone();
    copy_after.phase = ShengjiPhaseView::BottomCopying {
        player: PlayerId(1),
        milliseconds_remaining: 9_900,
    };
    assert!(only_shengji_transient_progress_changed(
        Some(&copy_before),
        Some(&copy_after)
    ));
}

#[test]
fn summary_scores_are_ranked_from_high_to_low() {
    let scores = [
        PlayerScore {
            player: PlayerId(2),
            score: 20,
        },
        PlayerScore {
            player: PlayerId(1),
            score: 80,
        },
        PlayerScore {
            player: PlayerId(0),
            score: 80,
        },
    ];

    let ranked = sorted_summary_scores(&scores);

    assert_eq!(
        ranked
            .iter()
            .map(|score| (score.player, score.score))
            .collect::<Vec<_>>(),
        vec![(PlayerId(0), 80), (PlayerId(1), 80), (PlayerId(2), 20)]
    );
}

#[test]
fn shengji_settlement_names_every_score_band() {
    assert_eq!(shengji_settlement_outcome_for_score(0, 3, 2), "闲家大光");
    assert_eq!(shengji_settlement_outcome_for_score(39, 2, 2), "闲家小光");
    assert_eq!(shengji_settlement_outcome_for_score(40, 1, 2), "闲家脱贫");
    assert_eq!(shengji_settlement_outcome_for_score(80, 0, 2), "闲家上台");
    assert_eq!(shengji_settlement_outcome_for_score(120, 1, 2), "闲家升1级");
    assert_eq!(shengji_settlement_outcome_for_score(160, 2, 2), "闲家升2级");
    assert_eq!(shengji_settlement_outcome_for_score(59, 2, 3), "闲家小光");
    assert_eq!(shengji_settlement_outcome_for_score(60, 1, 3), "闲家脱贫");
    assert_eq!(shengji_settlement_outcome_for_score(120, 0, 3), "闲家上台");
    assert_eq!(shengji_settlement_outcome_for_score(180, 1, 3), "闲家升1级");
    assert_eq!(shengji_settlement_outcome_for_score(79, 2, 4), "闲家小光");
    assert_eq!(shengji_settlement_outcome_for_score(80, 1, 4), "闲家脱贫");
    assert_eq!(shengji_settlement_outcome_for_score(160, 0, 4), "闲家上台");
    assert_eq!(shengji_settlement_outcome_for_score(240, 1, 4), "闲家升1级");
}

#[test]
fn score_cards_spiral_accelerate_and_tidally_deform_into_the_target() {
    let source = Vec2::new(640.0, 360.0);
    let target = Vec2::new(120.0, 620.0);
    let start = vortex_card_pose(source, target, 0.0, 1.0);
    let middle = vortex_card_pose(source, target, 0.5, 1.0);
    let late = vortex_card_pose(source, target, 0.9, 1.0);
    let end = vortex_card_pose(source, target, 1.0, 1.0);

    assert_eq!(start.position, source);
    assert!((end.position - target).length() < 0.001);
    let first_half = source.distance(target) - middle.position.distance(target);
    let second_half = middle.position.distance(target) - end.position.distance(target);
    assert!(second_half > first_half);
    assert!(late.scale.x < late.scale.y * 0.25);
    assert!(end.scale.length() < 0.001);
    assert!(late.rotation.is_finite());
}

#[test]
fn play_effect_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(PlayEffectState::default());
    app.insert_resource(UiAssets::default());
    app.add_systems(
        Update,
        (
            advance_play_effect,
            animate_sequence_play_effect,
            animate_bomb_play_effect,
            animate_heaven_bomb_play_effect,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn shengji_hand_interaction_systems_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiState::default());
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(CardDragSelection::default());
    app.add_systems(
        Update,
        (
            handle_shengji_card_drag_selection,
            animate_shengji_hand_card_slots,
            sync_shengji_card_drag_preview,
            animate_shengji_hand_cards,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn shengji_card_release_refreshes_the_contextual_play_button() {
    let card = ShengjiCard::suited(0, ShengjiSuit::Spade, ShengjiRank::Ace);
    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Left);
    mouse.clear();
    mouse.release(MouseButton::Left);

    let mut app = App::new();
    app.insert_resource(mouse);
    app.insert_resource(UiState::default());
    app.insert_resource(CardDragSelection {
        active: true,
        anchor: 0,
        current: 0,
        select: true,
    });
    app.world_mut().spawn((
        Interaction::None,
        RelativeCursorPosition::default(),
        ShengjiHandCardSlot {
            card,
            index: 0,
            hand_len: 1,
            is_last: true,
            hover_amount: 0.0,
        },
    ));
    app.add_systems(Update, handle_shengji_card_drag_selection);

    app.update();

    let ui = app.world().resource::<UiState>();
    assert!(ui.shengji.selected.contains(&card));
    assert!(ui.dirty);
}
#[test]
fn shengji_private_bottom_button_occupies_its_own_chat_side_slot() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let parent = commands
            .spawn(Node {
                position_type: PositionType::Relative,
                ..default()
            })
            .id();
        add_chat_panel(
            &mut commands,
            parent,
            &ChatPanelState::default(),
            &assets,
            Some(false),
            Some(false),
            Some(true),
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut query = app.world_mut().query::<(&Node, &UiAction)>();
    let (node, _) = query
        .iter(app.world())
        .find(|(_, action)| matches!(action, UiAction::ToggleShengjiBuried))
        .expect("埋底者可以看到私有底牌按钮");
    assert_eq!(node.left, px(-32));
    assert_eq!(node.top, px(274));
    assert_eq!(node.width, px(32));
}

#[test]
fn available_previous_trick_button_keeps_a_neutral_border() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let parent = commands.spawn(Node::default()).id();
        add_chat_panel(
            &mut commands,
            parent,
            &ChatPanelState::default(),
            &assets,
            None,
            Some(true),
            None,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<(&UiAction, &BorderColor), With<Button>>();
    let (_, border) = query
        .iter(app.world())
        .find(|(action, _)| matches!(action, UiAction::ShowShengjiPreviousTrick))
        .expect("上轮按钮在首轮牌结束后应可点击");
    assert_eq!(*border, BorderColor::all(BORDER));
}

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
        current_player: None,
        trick: None,
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
