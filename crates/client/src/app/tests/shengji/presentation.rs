use super::prelude::*;

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
    app.insert_resource(ShengjiUiState::default());
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
    app.insert_resource(ShengjiUiState::default());
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

    let game_ui = app.world().resource::<ShengjiUiState>();
    assert!(game_ui.selected.contains(&card));
    assert!(app.world().resource::<UiState>().dirty);
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
        let actions = [
            ChatAuxiliaryAction {
                label: "上轮",
                action: None,
                highlighted: false,
            },
            ChatAuxiliaryAction {
                label: "底牌",
                action: Some(UiAction::Shengji(ShengjiUiAction::ToggleBuried)),
                highlighted: true,
            },
        ];
        add_chat_panel(
            &mut commands,
            parent,
            &ChatPanelState::default(),
            &assets,
            Some(false),
            &actions,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut query = app.world_mut().query::<(&Node, &UiAction)>();
    let (node, _) = query
        .iter(app.world())
        .find(|(_, action)| matches!(action, UiAction::Shengji(ShengjiUiAction::ToggleBuried)))
        .expect("埋底者可以看到私有底牌按钮");
    assert_eq!(node.left, px(-32));
    assert_eq!(node.top, px(274));
    assert_eq!(node.width, px(32));
}

#[test]
fn available_previous_trick_button_keeps_a_neutral_border() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let parent = commands.spawn(Node::default()).id();
        let actions = [ChatAuxiliaryAction {
            label: "上轮",
            action: Some(UiAction::Shengji(ShengjiUiAction::ShowPreviousTrick)),
            highlighted: false,
        }];
        add_chat_panel(
            &mut commands,
            parent,
            &ChatPanelState::default(),
            &assets,
            None,
            &actions,
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
        .find(|(action, _)| {
            matches!(
                action,
                UiAction::Shengji(ShengjiUiAction::ShowPreviousTrick)
            )
        })
        .expect("上轮按钮在首轮牌结束后应可点击");
    assert_eq!(*border, BorderColor::all(BORDER));
}
