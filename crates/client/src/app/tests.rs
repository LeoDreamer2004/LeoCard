use super::*;

#[test]
fn host_game_picker_validates_identity_and_port_before_opening() {
    let mut form = ConnectionForm::default();
    form.player_name = "房主".to_owned();
    form.host_port = "52300".to_owned();
    assert_eq!(validated_host_form(&form), Ok(("房主".to_owned(), 52300)));

    form.host_port = "0".to_owned();
    assert!(validated_host_form(&form).is_err());
    form.host_port = "52300".to_owned();
    form.player_name.clear();
    assert!(validated_host_form(&form).is_err());
}

#[test]
fn host_game_picker_lists_every_playable_game() {
    assert_eq!(
        HOST_GAME_CHOICES,
        [
            ("七鬼五二三", "放空大脑, 有牌就出", GameKind::QiGui523),
            ("德州扑克", "窝要验牌!", GameKind::TexasHoldem),
            ("升级", "神对手 or 猪队友", GameKind::Shengji),
            ("UNO", "最后一张，记得喊 UNO!", GameKind::Uno),
        ]
    );
}

#[test]
fn settings_update_box_exposes_update_and_github_actions() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        render_settings_modal(
            &mut commands,
            root,
            &ConnectionForm::default(),
            &UpdateManager::default(),
            &assets,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut buttons = app
        .world_mut()
        .query_filtered::<(&UiAction, &ButtonTint), With<Button>>();
    let (_, tint) = buttons
        .iter(app.world())
        .find(|(action, _)| matches!(action, UiAction::StartUpdate))
        .expect("游戏设置应包含自动更新按钮");
    assert_eq!(tint.normal, READY);
    let github_buttons = app
        .world_mut()
        .query_filtered::<&UiAction, (With<Button>, With<GitHubRepositoryButton>)>()
        .iter(app.world())
        .filter(|action| matches!(action, UiAction::OpenGitHubRepository))
        .count();
    assert_eq!(github_buttons, 1);
    assert_eq!(
        GITHUB_REPOSITORY_URL,
        "https://github.com/LeoDreamer2004/LeoCard"
    );
}

#[test]
fn completed_update_dialog_offers_restart_and_later_actions() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        let mut updater = UpdateManager::default();
        updater.state = UpdateState::Ready {
            version: semver::Version::new(1, 2, 3),
            staged: PathBuf::from("leocard.update"),
        };
        updater.dialog_open = true;
        render_update_dialog(&mut commands, root, &updater, &assets);
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let actions = app
        .world_mut()
        .query::<&UiAction>()
        .iter(app.world())
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, UiAction::RestartToUpdate))
    );
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, UiAction::HideUpdateDialog))
    );
    assert!(
        actions
            .iter()
            .all(|action| !matches!(action, UiAction::OpenGitHubRepository))
    );
    let github_buttons = app
        .world_mut()
        .query_filtered::<&UiAction, (With<Button>, With<GitHubRepositoryButton>)>()
        .iter(app.world())
        .count();
    assert_eq!(github_buttons, 0);
}

#[test]
fn local_profile_applies_each_finished_match_once() {
    let identity = PlayerIdentity::from_secret_bytes([7; 32]);
    let profile_id = identity.profile_id();
    let mut profile = LocalPlayerProfile {
        identity,
        rating: PlayerRatingProfile {
            reference_points: 10,
            completed_games: 4,
            applied_matches: HashSet::new(),
            last_change: None,
        },
        game_profiles: PlayerGameProfiles::default(),
    };
    let match_id = MatchId([3; 16]);
    let changes = [leocard_protocol::PlayerReferenceChange {
        player: PlayerId(0),
        profile_id,
        delta: 3,
    }];

    assert!(profile.apply_finished_match(match_id, &changes));
    assert!(!profile.apply_finished_match(match_id, &changes));
    assert_eq!(profile.rating.reference_points, 13);
    assert_eq!(profile.rating.completed_games, 5);
    assert_eq!(profile.rating.last_change, Some((match_id, 3)));
}

#[test]
fn reference_levels_use_the_declared_boundaries() {
    assert_eq!(reference_level(1_001), "下界合金");
    assert_eq!(reference_level(1_000), "钻石");
    assert_eq!(reference_level(500), "钻石");
    assert_eq!(reference_level(499), "金");
    assert_eq!(reference_level(200), "金");
    assert_eq!(reference_level(199), "红石");
    assert_eq!(reference_level(100), "红石");
    assert_eq!(reference_level(99), "铁");
    assert_eq!(reference_level(50), "铁");
    assert_eq!(reference_level(49), "铜");
    assert_eq!(reference_level(10), "铜");
    assert_eq!(reference_level(9), "圆石");
    assert_eq!(reference_level(0), "圆石");
    assert_eq!(reference_level(-1), "木头");
    assert_eq!(reference_level(-10), "木头");
    assert_eq!(reference_level(-11), "泥土");
    assert_eq!(reference_level(-50), "泥土");
    assert_eq!(reference_level(-51), "堆肥桶");
    assert_eq!(reference_points_label(500), "等级:钻石  分数:500");
}

#[test]
fn interaction_menu_offers_the_selected_players_full_profile() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        add_interaction_menu(
            &mut commands,
            root,
            PlayerId(3),
            SeatSide::Top,
            "远端玩家",
            None,
            500,
            37,
            &PlayerGameProfiles::default(),
            &assets,
        );
    }

    let mut assets = UiAssets::default();
    for kind in [
        PlayerInteractionKind::Flower,
        PlayerInteractionKind::Egg,
        PlayerInteractionKind::Wine,
        PlayerInteractionKind::Shoe,
    ] {
        assets
            .interaction_images
            .insert((kind, false), Handle::default());
    }
    let mut app = App::new();
    app.insert_resource(assets);
    app.add_systems(Startup, setup);
    app.update();

    let profiles = app
        .world_mut()
        .query::<&UiAction>()
        .iter(app.world())
        .filter_map(|action| match action {
            UiAction::OpenPlayerProfile(profile) => Some(profile),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].name, "远端玩家");
    assert_eq!(profiles[0].reference_points, 500);
    assert_eq!(profiles[0].completed_games, 37);
    assert_eq!(profiles[0].game_profiles, PlayerGameProfiles::default());
}

#[test]
fn full_profile_modal_renders_the_selected_players_statistics() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let root = commands.spawn(Node::default()).id();
        let game_profiles = PlayerGameProfiles {
            interactions: Some(PlayerInteractionStats {
                flowers_received: 21,
                eggs_received: 12,
            }),
            ..PlayerGameProfiles::default()
        };
        render_profile_modal(
            &mut commands,
            root,
            "远端玩家",
            None,
            500,
            37,
            &game_profiles,
            ProfileGameTab::Uno,
            &assets,
        );
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let labels = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|text| text.0.as_str())
        .collect::<Vec<_>>();
    assert!(labels.contains(&"远端玩家"));
    assert!(labels.contains(&"500"));
    assert!(labels.contains(&"37"));
    assert!(labels.contains(&"钻石"));
    assert!(labels.contains(&"21"));
    assert!(labels.contains(&"12"));
    assert!(!labels.contains(&"玩家头像"));
    assert!(!labels.contains(&"玩家名称"));
    assert!(!labels.iter().any(|label| label.starts_with("当前等级：")));
    for label in ["七鬼五二三", "德州扑克", "升级", "UNO"] {
        assert!(labels.contains(&label));
    }

    let tab_actions = app
        .world_mut()
        .query::<&UiAction>()
        .iter(app.world())
        .filter(|action| matches!(action, UiAction::SelectProfileGameTab(_)))
        .count();
    assert_eq!(tab_actions, 4);
    let mut selected_tab = app
        .world_mut()
        .query_filtered::<&UiAction, With<SelectedProfileGameTab>>();
    assert!(matches!(
        selected_tab.single(app.world()).unwrap(),
        UiAction::SelectProfileGameTab(ProfileGameTab::Uno)
    ));
    let mut game_tabs = app
        .world_mut()
        .query_filtered::<&Node, With<ProfileGameTabButton>>();
    let game_tabs = game_tabs.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(game_tabs.len(), 4);
    assert!(
        game_tabs
            .iter()
            .all(|node| node.flex_basis == px(0) && node.flex_grow == 1.0)
    );
    let mut stats = app
        .world_mut()
        .query_filtered::<(&Node, Option<&BackgroundColor>), With<ProfileStat>>();
    let stats = stats.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(stats.len(), 3);
    assert!(stats.iter().all(|(node, background)| {
        node.flex_basis == px(0)
            && node.flex_grow == 1.0
            && node.border == UiRect::ZERO
            && background.is_none_or(|background| background.0 == Color::NONE)
    }));
    let mut columns = app
        .world_mut()
        .query_filtered::<&Node, With<ProfileGameColumn>>();
    assert_eq!(columns.iter(app.world()).count(), 4);
    let mut content = app
        .world_mut()
        .query_filtered::<&Children, With<ProfileGameContent>>();
    assert_eq!(content.single(app.world()).unwrap().len(), 4);
}

#[test]
fn qigui523_profile_rows_use_placeholders_for_legacy_profiles_and_format_statistics() {
    let legacy = qigui523_profile_rows(None);
    assert_eq!(legacy.len(), 18);
    assert!(legacy.iter().all(|(_, value)| value == "--"));

    let rows = qigui523_profile_rows(Some(&QiGui523ProfileStats {
        completed_games: 4,
        total_score: 500,
        total_reference_delta: 6,
        placement_counts: [2, 1, 1, 0, 0, 0],
        straight_plays: 9,
        consecutive_pair_plays: 8,
        airplane_plays: 7,
        bomb_plays: 6,
        heaven_bomb_plays: 5,
        longest_straight: 11,
        longest_consecutive_pairs: 4,
        longest_airplane: 3,
    }));
    let value = |label| {
        rows.iter()
            .find_map(|(candidate, value)| (*candidate == label).then_some(value.as_str()))
            .unwrap()
    };
    assert_eq!(value("对局数"), "4");
    assert_eq!(value("分数增减"), "+1.5");
    assert_eq!(value("场得分"), "125.0");
    assert_eq!(value("平均顺位"), "1.75");
    assert_eq!(value("一位率"), "50.0%");
    assert_eq!(value("飞机最长长度"), "3");
}

#[test]
fn texas_holdem_profile_rows_format_combined_long_term_statistics() {
    let legacy = texas_holdem_profile_rows(None);
    assert_eq!(legacy.len(), 25);
    assert!(legacy.iter().all(|(_, value)| value == "--"));

    let rows = texas_holdem_profile_rows(Some(&TexasHoldemProfileStats {
        completed_games: 4,
        total_reference_delta: -6,
        total_final_chips: 500,
        placement_counts: [2, 1, 1, 0, 0, 0],
        wagered_chips: 90,
        wager_actions: 6,
        voluntary_actions: 20,
        check_actions: 5,
        raise_actions: 4,
        all_in_actions: 2,
        hands_played: 10,
        hands_folded: 3,
        hand_category_counts: [8, 7, 6, 5, 4, 3, 2, 1, 1, 1],
    }));
    let value = |label| {
        rows.iter()
            .find_map(|(candidate, value)| (*candidate == label).then_some(value.as_str()))
            .unwrap()
    };
    assert_eq!(value("对局数"), "4");
    assert_eq!(value("分数增减"), "-1.5");
    assert_eq!(value("场筹码"), "125.0");
    assert_eq!(value("平均顺位"), "1.75");
    assert_eq!(value("一位率"), "50.0%");
    assert_eq!(value("平均加注"), "15.0");
    assert_eq!(value("过牌率"), "25.0%");
    assert_eq!(value("加注率"), "20.0%");
    assert_eq!(value("全下率"), "10.0%");
    assert_eq!(value("局弃牌率"), "30.0%");
    assert_eq!(value("高牌次数"), "8");
    assert_eq!(value("皇家同花顺次数"), "1");
}

#[test]
fn shengji_profile_rows_format_role_rates_and_play_statistics() {
    let legacy = shengji_profile_rows(None);
    assert_eq!(legacy.len(), 21);
    assert!(legacy.iter().all(|(_, value)| value == "--"));

    let rows = shengji_profile_rows(Some(&ShengjiProfileStats {
        completed_games: 8,
        total_reference_delta: 12,
        dealer_team_games: 4,
        dealer_team_score: 160,
        collecting_team_games: 4,
        collecting_team_score: 480,
        dealer_games: 2,
        declaration_games: 2,
        counter_games: 1,
        defended_kitty_games: 3,
        captured_kitty_games: 2,
        buried_games: 2,
        buried_points: 30,
        plays: 40,
        winning_plays: 10,
        crossing_games: 1,
        play_category_counts: [7, 5, 4, 3, 2],
        longest_tractor: 4,
        longest_titanic: 3,
        longest_space_fortress: 2,
        longest_throw: 12,
    }));
    let value = |label| {
        rows.iter()
            .find_map(|(candidate, value)| (*candidate == label).then_some(value.as_str()))
            .unwrap()
    };
    assert_eq!(value("分数增减"), "+1.5");
    assert_eq!(value("庄场得分"), "40.0");
    assert_eq!(value("闲场得分"), "120.0");
    assert_eq!(value("坐庄率"), "25.0%");
    assert_eq!(value("保底率"), "75.0%");
    assert_eq!(value("扣底率"), "50.0%");
    assert_eq!(value("底牌平均分"), "15.0");
    assert_eq!(value("牌权率"), "25.0%");
    assert_eq!(value("太空堡垒次数"), "3");
    assert_eq!(value("最长甩牌长度"), "12");
}

#[test]
fn uno_profile_rows_format_rank_penalty_and_action_statistics() {
    let legacy = uno_profile_rows(None);
    assert_eq!(legacy.len(), 21);
    assert!(legacy.iter().all(|(_, value)| value == "--"));

    let rows = uno_profile_rows(Some(&UnoProfileStats {
        completed_games: 4,
        total_reference_delta: 6,
        total_remaining_score: 180,
        placement_counts: [2, 1, 1, 0, 0, 0],
        max_hand_cards: 17,
        max_penalty_cards: 8,
        max_skipped_turns: 3,
        uno_calls: 9,
        uno_penalties: 2,
        challenges: 4,
        successful_challenges: 3,
        challenges_received: 5,
        successful_challenges_received: 2,
        jump_in_opportunities: 10,
        successful_jump_ins: 6,
    }));
    let value = |label| {
        rows.iter()
            .find_map(|(candidate, value)| (*candidate == label).then_some(value.as_str()))
            .unwrap()
    };
    assert_eq!(value("对局数"), "4");
    assert_eq!(value("分数增减"), "+1.5");
    assert_eq!(value("场剩余分数"), "45.0");
    assert_eq!(value("平均顺位"), "1.75");
    assert_eq!(value("一位率"), "50.0%");
    assert_eq!(value("最多牌数"), "17");
    assert_eq!(value("质疑成功率"), "75.0%");
    assert_eq!(value("被质疑成功率"), "40.0%");
    assert_eq!(value("抢出次数"), "6");
    assert_eq!(value("抢出成功率"), "60.0%");
}

#[test]
fn pre_detailed_profile_decodes_with_unknown_qigui523_statistics() {
    let previous = PreDetailedStoredPlayerProfile {
        secret_key: [9; 32],
        games: PreDetailedStoredGameProfiles {
            qigui523: PreDetailedStoredRatingProfile {
                reference_points: 321,
                completed_games: 17,
                applied_matches: vec![MatchId([4; 16])],
            },
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();

    let decoded = decode_player_profile(&bytes).unwrap();

    assert_eq!(decoded.secret_key, [9; 32]);
    assert_eq!(decoded.games.qigui523.reference_points, 321);
    assert_eq!(decoded.games.qigui523.completed_games, 17);
    assert_eq!(
        decoded.games.qigui523.applied_matches,
        vec![MatchId([4; 16])]
    );
    assert_eq!(decoded.games.qigui523.qigui523_stats, None);
    assert_eq!(decoded.games.texas_holdem_stats, None);
    assert_eq!(decoded.games.shengji_stats, None);
    assert_eq!(decoded.games.uno_stats, None);
    assert_eq!(decoded.games.interaction_stats, None);
}

#[test]
fn pre_texas_profile_preserves_qigui523_details_and_marks_texas_unknown() {
    let qigui523_stats = QiGui523ProfileStats {
        completed_games: 2,
        total_score: 88,
        total_reference_delta: 4,
        ..QiGui523ProfileStats::default()
    };
    let previous = PreTexasStoredPlayerProfile {
        secret_key: [7; 32],
        games: PreTexasStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 42,
                completed_games: 2,
                applied_matches: vec![MatchId([8; 16])],
                qigui523_stats: Some(qigui523_stats.clone()),
            },
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();

    let decoded = decode_player_profile(&bytes).unwrap();

    assert_eq!(decoded.games.qigui523.qigui523_stats, Some(qigui523_stats));
    assert_eq!(decoded.games.texas_holdem_stats, None);
    assert_eq!(decoded.games.shengji_stats, None);
    assert_eq!(decoded.games.uno_stats, None);
    assert_eq!(decoded.games.interaction_stats, None);
}

#[test]
fn pre_shengji_profile_preserves_existing_game_details() {
    let texas_holdem_stats = TexasHoldemProfileStats {
        completed_games: 3,
        total_final_chips: 300,
        ..TexasHoldemProfileStats::default()
    };
    let previous = PreShengjiStoredPlayerProfile {
        secret_key: [6; 32],
        games: PreShengjiStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 8,
                completed_games: 3,
                applied_matches: Vec::new(),
                qigui523_stats: None,
            },
            texas_holdem_stats: Some(texas_holdem_stats.clone()),
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();

    let decoded = decode_player_profile(&bytes).unwrap();

    assert_eq!(decoded.games.texas_holdem_stats, Some(texas_holdem_stats));
    assert_eq!(decoded.games.shengji_stats, None);
    assert_eq!(decoded.games.uno_stats, None);
    assert_eq!(decoded.games.interaction_stats, None);
}

#[test]
fn pre_uno_profile_preserves_existing_game_details() {
    let shengji_stats = ShengjiProfileStats {
        completed_games: 5,
        declaration_games: 2,
        ..ShengjiProfileStats::default()
    };
    let previous = PreUnoStoredPlayerProfile {
        secret_key: [5; 32],
        games: PreUnoStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 12,
                completed_games: 5,
                applied_matches: Vec::new(),
                qigui523_stats: None,
            },
            texas_holdem_stats: None,
            shengji_stats: Some(shengji_stats.clone()),
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();

    let decoded = decode_player_profile(&bytes).unwrap();

    assert_eq!(decoded.games.shengji_stats, Some(shengji_stats));
    assert_eq!(decoded.games.uno_stats, None);
    assert_eq!(decoded.games.interaction_stats, None);
}

#[test]
fn pre_interaction_profile_preserves_uno_statistics_and_marks_interactions_unknown() {
    let uno_stats = UnoProfileStats {
        completed_games: 6,
        uno_calls: 8,
        ..UnoProfileStats::default()
    };
    let previous = PreInteractionStoredPlayerProfile {
        secret_key: [4; 32],
        games: PreInteractionStoredGameProfiles {
            qigui523: StoredRatingProfile {
                reference_points: 14,
                completed_games: 6,
                applied_matches: Vec::new(),
                qigui523_stats: None,
            },
            texas_holdem_stats: None,
            shengji_stats: None,
            uno_stats: Some(uno_stats.clone()),
        },
    };
    let bytes = postcard::to_allocvec(&previous).unwrap();

    let decoded = decode_player_profile(&bytes).unwrap();

    assert_eq!(decoded.games.uno_stats, Some(uno_stats));
    assert_eq!(decoded.games.interaction_stats, None);
}

#[test]
fn not_players_turn_rejection_does_not_create_a_popup() {
    assert_eq!(
        rejection_label(&RejectReason::GameViolation(GameViolation::QiGui523(
            RuleViolation::NotPlayersTurn,
        ))),
        None
    );
    assert!(
        rejection_label(&RejectReason::GameViolation(GameViolation::QiGui523(
            RuleViolation::InvalidPattern,
        )))
        .is_some()
    );
}

#[test]
fn drag_selection_range_works_in_both_directions() {
    let mut drag = CardDragSelection {
        active: true,
        anchor: 5,
        current: 2,
        select: true,
    };
    assert!(!drag.contains(1));
    assert!(drag.contains(2));
    assert!(drag.contains(4));
    assert!(drag.contains(5));
    assert!(!drag.contains(6));

    drag.active = false;
    assert!(!drag.contains(4));
}

#[test]
fn ui_scale_fits_design_size_and_respects_manual_zoom() {
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 1.0), 1.0);
    assert_eq!(calculate_ui_scale(640.0, 400.0, 1.0), 0.5);
    assert_eq!(calculate_ui_scale(2560.0, 1440.0, 1.0), 2.0);
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 1.5), 1.5);
    assert_eq!(calculate_ui_scale(1280.0, 720.0, 10.0), 1.5);
}

#[test]
fn table_brightness_is_normalized_and_mapped_to_the_slider() {
    assert_eq!(MIN_TABLE_BRIGHTNESS, 0.1);
    assert_eq!(normalize_table_brightness(0.0), 1.0);
    assert_eq!(normalize_table_brightness(f32::NAN), 1.0);
    assert_eq!(table_brightness_fraction(MIN_TABLE_BRIGHTNESS), 0.0);
    assert_eq!(table_brightness_fraction(MAX_TABLE_BRIGHTNESS), 1.0);
    assert_eq!(slider_fraction_from_relative_x(-0.5), 0.0);
    assert_eq!(slider_fraction_from_relative_x(0.0), 0.5);
    assert_eq!(slider_fraction_from_relative_x(0.5), 1.0);
    assert_eq!(slider_fraction_from_relative_x(-2.0), 0.0);
    assert_eq!(slider_fraction_from_relative_x(2.0), 1.0);
}

#[test]
fn table_material_parameters_clamp_invalid_visual_settings() {
    let params = table_material_params(f32::NAN, 5.0, true);
    assert_eq!(params.x, DEFAULT_TABLE_VIGNETTE);
    assert_eq!(params.y, 1.0);
    assert_eq!(params.z, 1.0);
    assert_eq!(params.w, 0.0);

    let custom = table_material_params(1.0, DEFAULT_TABLE_VIGNETTE, false);
    assert_eq!(custom.z, 0.0);
}

#[test]
fn table_background_shader_tiles_builtin_felt_and_cover_crops_custom_images() {
    let shader = include_str!("../../../../assets/shaders/table_background.wgsl");
    assert!(shader.contains("let cover_scale = max("));
    assert!(shader.contains("let crop_origin ="));
    assert!(shader.contains("return fract("));
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_accepts_compact_cards_and_assigns_physical_copies() {
    let cards = parse_developer_hand("s4 H5,c6;dK ST").unwrap();
    assert_eq!(
        cards,
        vec![
            Card::suited(0, Suit::Spade, Rank::Four),
            Card::suited(0, Suit::Heart, Rank::Five),
            Card::suited(0, Suit::Club, Rank::Six),
            Card::suited(0, Suit::Diamond, Rank::King),
            Card::suited(0, Suit::Spade, Rank::Ten),
        ]
    );

    let copies = parse_developer_hand("S4S4S4").unwrap();
    assert_eq!(copies[0].deck(), 0);
    assert_eq!(copies[1].deck(), 1);
    assert_eq!(copies[2].deck(), 2);
    assert!(parse_developer_hand("S10").is_err());
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_supports_red_and_black_jokers() {
    assert_eq!(
        parse_developer_hand("RJBJ").unwrap(),
        vec![
            Card::suited(0, Suit::Spade, Rank::Joker),
            Card::suited(0, Suit::Club, Rank::Joker),
        ]
    );
    assert!(parse_developer_hand("S0C0").is_err());
    assert_eq!(
        parse_developer_hand("SJCJ").unwrap(),
        vec![
            Card::suited(0, Suit::Spade, Rank::Jack),
            Card::suited(0, Suit::Club, Rank::Jack),
        ]
    );
    assert!(parse_developer_hand("SJOKER").is_err());
    let small_jokers = parse_developer_hand("BJBJ").unwrap();
    assert_eq!(small_jokers[0], Card::suited(0, Suit::Club, Rank::Joker));
    assert_eq!(small_jokers[1], Card::suited(1, Suit::Club, Rank::Joker));
}

#[cfg(feature = "developer")]
#[test]
fn developer_hand_parser_randomizes_suits_in_rank_only_mode() {
    let cards = parse_developer_hand("70523").unwrap();
    assert_eq!(
        cards.iter().copied().map(Card::rank).collect::<Vec<_>>(),
        vec![Rank::Seven, Rank::Joker, Rank::Five, Rank::Two, Rank::Three,]
    );
    assert!(matches!(cards[1].suit(), Suit::Spade | Suit::Club));
    assert_eq!(cards.iter().copied().collect::<HashSet<_>>().len(), 5);
    let repeated = parse_developer_hand("77777000").unwrap();
    assert_eq!(repeated.len(), 8);
    assert_eq!(
        repeated.iter().copied().collect::<HashSet<_>>().len(),
        repeated.len()
    );
}

#[test]
fn every_card_maps_to_an_existing_asset() {
    assert_eq!(
        card_asset_path(Rank::Ace, Suit::Spade),
        "vendor/kenney/boardgame/PNG/Cards/cardSpadesA.png"
    );
    assert_eq!(
        card_asset_path(Rank::Ten, Suit::Diamond),
        "vendor/kenney/boardgame/PNG/Cards/cardDiamonds10.png"
    );
    assert_eq!(
        card_asset_path(Rank::Joker, Suit::Club),
        "vendor/kenney/boardgame/PNG/Cards/cardJoker.png"
    );
    assert_eq!(
        card_asset_path(Rank::Joker, Suit::Spade),
        "vendor/kenney/boardgame/PNG/Cards/cardJokerBig.png"
    );

    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    for card in build_deck(1) {
        let path = asset_root.join(card_asset_path(card.rank(), card.suit()));
        assert!(path.is_file(), "missing card asset: {}", path.display());
    }
    assert!(asset_root.join(TABLE_FELT_ASSET).is_file());
    assert!(asset_root.join(UI_FONT_ASSET).is_file());
    assert!(asset_root.join("fonts/OFL-ChillRoundGothic.txt").is_file());
    assert!(asset_root.join("icons/list-menu.png").is_file());
    assert!(asset_root.join("icons/github-mark.png").is_file());
    assert!(
        asset_root
            .join("ui/effects/sequence_airplane.png")
            .is_file()
    );
    assert!(
        asset_root
            .join("ui/effects/sequence_airplane.svg")
            .is_file()
    );
    assert!(asset_root.join("ui/effects/shengji_target.png").is_file());
    assert!(asset_root.join("ui/effects/shengji_dart.png").is_file());
}

#[test]
fn every_runtime_ui_and_card_sound_decodes_with_enabled_bevy_formats() {
    use bevy::audio::Decodable;

    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let mut sounds = vec![
        "vendor/kenney/ui/Sounds/click-a.ogg".to_owned(),
        "vendor/kenney/ui/Sounds/click-b.ogg".to_owned(),
        "vendor/kenney/interface-sounds/Audio/error_007.ogg".to_owned(),
    ];
    for (prefix, count) in [
        ("card-slide", 8),
        ("card-place", 4),
        ("card-shove", 4),
        ("card-fan", 2),
        ("chip-lay", 3),
        ("chips-collide", 4),
        ("chips-handle", 6),
        ("chips-stack", 6),
    ] {
        sounds.extend(
            (1..=count)
                .map(|index| format!("vendor/kenney/casino-audio/Audio/{prefix}-{index}.ogg")),
        );
    }
    sounds.extend(["tap-a.ogg", "tap-b.ogg"].map(|name| format!("vendor/kenney/ui/Sounds/{name}")));
    sounds.extend(
        [
            "bong_001.ogg",
            "drop_004.ogg",
            "switch_003.ogg",
            "switch_004.ogg",
            "pluck_001.ogg",
            "pluck_002.ogg",
            "confirmation_001.ogg",
            "confirmation_002.ogg",
            "confirmation_003.ogg",
            "confirmation_004.ogg",
            "scratch_004.ogg",
        ]
        .map(|name| format!("vendor/kenney/interface-sounds/Audio/{name}")),
    );
    for (prefix, count) in [
        ("close", 4),
        ("error", 8),
        ("glass", 6),
        ("maximize", 9),
        ("question", 4),
        ("scroll", 5),
        ("select", 8),
    ] {
        sounds.extend(
            (1..=count).map(|index| {
                format!("vendor/kenney/interface-sounds/Audio/{prefix}_00{index}.ogg")
            }),
        );
    }
    sounds.push("vendor/noname/audio/effect/flappybird_start.ogg".to_owned());
    sounds.push("vendor/noname/audio/effect/flappybird_score.ogg".to_owned());
    sounds.push("vendor/noname/audio/effect/flappybird_die.ogg".to_owned());
    sounds.push("audio/shengji/power-off.ogg".to_owned());
    sounds.push("audio/shengji/power-on.ogg".to_owned());
    sounds.extend(
        (0..QUICK_VOICE_COUNT).map(|index| format!("vendor/noname/voice/male/{index}.mp3")),
    );
    sounds.push("vendor/noname/damage_fire2.mp3".to_owned());

    for relative in sounds {
        let path = asset_root.join(&relative);
        let source = AudioSource {
            bytes: std::fs::read(&path).unwrap().into(),
        };
        let mut decoder = source.decoder();
        assert!(
            decoder.next().is_some(),
            "audio contains no decodable samples: {}",
            path.display()
        );
    }
}

#[test]
fn chat_input_obeys_the_protocol_character_limit() {
    let mut input = "你好".to_owned();
    append_chat_input(&mut input, &"界".repeat(MAX_CHAT_MESSAGE_CHARS));
    assert_eq!(input.chars().count(), MAX_CHAT_MESSAGE_CHARS);
    assert_eq!(QUICK_VOICES.len(), usize::from(QUICK_VOICE_COUNT));
}

#[test]
fn shengji_bidding_countdown_rounds_up_to_whole_seconds() {
    assert_eq!(shengji_bidding_countdown_label(5_000), "5秒");
    assert_eq!(shengji_bidding_countdown_label(4_001), "5秒");
    assert_eq!(shengji_bidding_countdown_label(4_000), "4秒");
    assert_eq!(shengji_bidding_countdown_label(1), "1秒");
    assert_eq!(shengji_bidding_countdown_label(0), "0秒");
}

#[test]
fn avatar_is_cropped_and_encoded_as_bounded_64px_png() {
    let source = image::DynamicImage::ImageRgba8(image::ImageBuffer::from_pixel(
        80,
        40,
        image::Rgba([210, 80, 30, 255]),
    ));
    let mut encoded = Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();

    let normalized = normalize_avatar_bytes(&encoded.into_inner()).unwrap();
    let decoded =
        image::load_from_memory_with_format(&normalized, image::ImageFormat::Png).unwrap();

    assert_eq!(decoded.width(), AVATAR_DIMENSION);
    assert_eq!(decoded.height(), AVATAR_DIMENSION);
    assert!(valid_normalized_avatar(&normalized));
    assert!(normalized.len() <= MAX_AVATAR_BYTES);
}

#[test]
fn jpeg_avatar_is_accepted_and_normalized_to_png() {
    let source = image::DynamicImage::ImageRgb8(image::ImageBuffer::from_pixel(
        48,
        80,
        image::Rgb([35, 140, 210]),
    ));
    let mut encoded = Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Jpeg)
        .unwrap();

    let normalized = normalize_avatar_bytes(&encoded.into_inner()).unwrap();
    let decoded =
        image::load_from_memory_with_format(&normalized, image::ImageFormat::Png).unwrap();

    assert!(normalized.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(decoded.width(), AVATAR_DIMENSION);
    assert_eq!(decoded.height(), AVATAR_DIMENSION);
    assert!(valid_normalized_avatar(&normalized));
    assert!(normalized.len() <= MAX_AVATAR_BYTES);
}

#[test]
fn jpeg_table_felt_is_decoded_without_converting_the_saved_path() {
    let source = image::DynamicImage::ImageRgb8(image::ImageBuffer::from_pixel(
        96,
        54,
        image::Rgb([18, 72, 48]),
    ));
    let mut encoded = Cursor::new(Vec::new());
    source
        .write_to(&mut encoded, image::ImageFormat::Jpeg)
        .unwrap();

    let decoded = decode_table_felt_image(&encoded.into_inner()).unwrap();
    assert_eq!(decoded.width(), 96);
    assert_eq!(decoded.height(), 54);
}

#[test]
fn preferences_round_trip_including_avatar() {
    let saved = SavedPreferences {
        global: GlobalPreferences {
            player_name: "测试玩家".to_owned(),
            avatar_png: Some(vec![1, 2, 3]),
            host_port: "52301".to_owned(),
            join_address: "192.168.1.2:52302".to_owned(),
            table_felt_path: Some(PathBuf::from("/tmp/table-felt.png")),
            table_brightness: 0.75,
            table_vignette: 0.42,
            audio_volume: 0.63,
        },
        games: GamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: RuleSet {
                    deck_count: 3,
                    hand_size: 12,
                    time_control: TimeControl::ThirtyPlusSixty,
                    advanced_play_types: true,
                    ..normalize_host_rules(RuleSet::default())
                },
            },
            texas_holdem: TexasHoldemPreferences {
                host_rules: TexasHoldemRuleSet {
                    starting_chips: 40,
                    short_deck: true,
                    ignore_kickers: true,
                    omaha: true,
                    ..normalize_texas_holdem_rules(TexasHoldemRuleSet::default())
                },
            },
            shengji: ShengjiPreferences {
                host_rules: ShengjiRuleSet {
                    deck_count: 4,
                    bottom_copy: true,
                    five_trump_crossing: true,
                    constant_trump: true,
                    ..ShengjiRuleSet::default()
                },
            },
            uno: UnoPreferences {
                host_rules: UnoRuleSet {
                    stack_draw_four_on_draw_two: true,
                    uno_callout: false,
                    skip_draw_penalty: true,
                    stack_skip: true,
                    jump_in: true,
                },
            },
        },
    };
    let encoded = postcard::to_allocvec(&saved).unwrap();
    let decoded: SavedPreferences = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded.global.player_name, saved.global.player_name);
    assert_eq!(decoded.global.avatar_png, saved.global.avatar_png);
    assert_eq!(decoded.global.host_port, saved.global.host_port);
    assert_eq!(decoded.global.join_address, saved.global.join_address);
    assert_eq!(
        decoded.games.texas_holdem.host_rules,
        saved.games.texas_holdem.host_rules
    );
    assert_eq!(decoded.global.table_felt_path, saved.global.table_felt_path);
    assert_eq!(
        decoded.global.table_brightness,
        saved.global.table_brightness
    );
    assert_eq!(decoded.global.table_vignette, saved.global.table_vignette);
    assert_eq!(decoded.global.audio_volume, saved.global.audio_volume);
    assert_eq!(
        decoded.games.qigui523.host_rules,
        saved.games.qigui523.host_rules
    );
    assert_eq!(
        decoded.games.shengji.host_rules,
        saved.games.shengji.host_rules
    );
    assert_eq!(decoded.games.uno.host_rules, saved.games.uno.host_rules);
}

#[test]
fn pre_omaha_preferences_keep_texas_rules_and_disable_omaha() {
    let previous = PreOmahaSavedPreferences {
        global: GlobalPreferences {
            player_name: "奥马哈前版本玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: PreOmahaGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: normalize_host_rules(RuleSet::default()),
            },
            texas_holdem: PreOmahaTexasHoldemPreferences {
                host_rules: PreOmahaTexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    starting_chips: 40,
                    short_deck: true,
                    ignore_kickers: true,
                },
            },
            shengji: ShengjiPreferences::default(),
            uno: UnoPreferences::default(),
        },
    };

    let decoded = decode_preferences(&postcard::to_allocvec(&previous).unwrap()).unwrap();
    let rules = decoded.games.texas_holdem.host_rules;
    assert_eq!(decoded.global.player_name, "奥马哈前版本玩家");
    assert_eq!(rules.starting_chips, 40);
    assert!(rules.short_deck);
    assert!(rules.ignore_kickers);
    assert!(!rules.omaha);
}

#[test]
fn pre_uno_preferences_gain_default_uno_rules() {
    let previous = PreUnoSavedPreferences {
        global: GlobalPreferences {
            player_name: "UNO 前版本玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: PreUnoGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: normalize_host_rules(RuleSet::default()),
            },
            texas_holdem: PreOmahaTexasHoldemPreferences {
                host_rules: PreOmahaTexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    starting_chips: TexasHoldemRuleSet::default().starting_chips,
                    short_deck: false,
                    ignore_kickers: false,
                },
            },
            shengji: ShengjiPreferences::default(),
        },
    };
    let encoded = postcard::to_allocvec(&previous).unwrap();
    let decoded = decode_preferences(&encoded).unwrap();
    assert_eq!(decoded.global.player_name, "UNO 前版本玩家");
    assert_eq!(decoded.games.uno.host_rules, UnoRuleSet::default());
}

#[test]
fn pre_jump_in_preferences_preserve_uno_rules_and_disable_jump_in() {
    let previous = PreJumpInSavedPreferences {
        global: GlobalPreferences {
            player_name: "抢出前版本玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: PreJumpInGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: normalize_host_rules(RuleSet::default()),
            },
            texas_holdem: PreOmahaTexasHoldemPreferences {
                host_rules: PreOmahaTexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    starting_chips: TexasHoldemRuleSet::default().starting_chips,
                    short_deck: false,
                    ignore_kickers: false,
                },
            },
            shengji: ShengjiPreferences::default(),
            uno: PreJumpInUnoPreferences {
                host_rules: PreJumpInUnoRuleSet {
                    stack_draw_four_on_draw_two: true,
                    uno_callout: false,
                    skip_draw_penalty: true,
                    stack_skip: true,
                },
            },
        },
    };
    let decoded = decode_preferences(&postcard::to_allocvec(&previous).unwrap()).unwrap();
    assert_eq!(decoded.global.player_name, "抢出前版本玩家");
    assert!(decoded.games.uno.host_rules.stack_draw_four_on_draw_two);
    assert!(!decoded.games.uno.host_rules.uno_callout);
    assert!(decoded.games.uno.host_rules.skip_draw_penalty);
    assert!(decoded.games.uno.host_rules.stack_skip);
    assert!(!decoded.games.uno.host_rules.jump_in);
}

#[test]
fn previous_uno_preferences_drop_configured_count_and_disable_skip_rules() {
    let previous = PreviousUnoSavedPreferences {
        global: GlobalPreferences {
            player_name: "旧 UNO 玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: PreviousUnoGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: normalize_host_rules(RuleSet::default()),
            },
            texas_holdem: PreOmahaTexasHoldemPreferences {
                host_rules: PreOmahaTexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    starting_chips: TexasHoldemRuleSet::default().starting_chips,
                    short_deck: false,
                    ignore_kickers: false,
                },
            },
            shengji: ShengjiPreferences::default(),
            uno: PreviousUnoPreferences {
                host_rules: PreviousUnoRuleSet {
                    player_count: 4,
                    stack_draw_four_on_draw_two: true,
                    uno_callout: true,
                },
            },
        },
    };
    let decoded = decode_preferences(&postcard::to_allocvec(&previous).unwrap()).unwrap();
    assert!(decoded.games.uno.host_rules.stack_draw_four_on_draw_two);
    assert!(decoded.games.uno.host_rules.uno_callout);
    assert!(!decoded.games.uno.host_rules.skip_draw_penalty);
    assert!(!decoded.games.uno.host_rules.stack_skip);
}

#[test]
fn previous_preferences_gain_disabled_kicker_rule_without_losing_games() {
    let previous = PreviousSavedPreferences {
        global: GlobalPreferences {
            player_name: "旧德州玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: PreviousGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: normalize_host_rules(RuleSet::default()),
            },
            texas_holdem: PreviousTexasHoldemPreferences {
                host_rules: PreviousTexasHoldemRuleSet {
                    player_count: 6,
                    starting_chips: 40,
                    short_deck: true,
                },
            },
            shengji: ShengjiPreferences {
                host_rules: ShengjiRuleSet {
                    deck_count: 4,
                    bottom_copy: true,
                    ..ShengjiRuleSet::default()
                },
            },
        },
    };
    let encoded = postcard::to_allocvec(&previous).unwrap();
    let decoded = decode_preferences(&encoded).unwrap();

    assert_eq!(decoded.global.player_name, "旧德州玩家");
    assert_eq!(decoded.games.texas_holdem.host_rules.starting_chips, 40);
    assert!(decoded.games.texas_holdem.host_rules.short_deck);
    assert!(!decoded.games.texas_holdem.host_rules.ignore_kickers);
    assert!(!decoded.games.texas_holdem.host_rules.omaha);
    assert_eq!(decoded.games.shengji.host_rules.deck_count, 4);
    assert!(decoded.games.shengji.host_rules.bottom_copy);
}

#[test]
fn legacy_preferences_gain_default_shengji_rules_without_losing_existing_values() {
    let legacy = LegacySavedPreferences {
        global: GlobalPreferences {
            player_name: "旧版玩家".to_owned(),
            avatar_png: None,
            host_port: "52301".to_owned(),
            join_address: "127.0.0.1:52301".to_owned(),
            table_felt_path: None,
            table_brightness: 0.8,
            table_vignette: 0.3,
            audio_volume: 0.8,
        },
        games: LegacyGamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: RuleSet {
                    deck_count: 3,
                    ..normalize_host_rules(RuleSet::default())
                },
            },
            texas_holdem: PreviousTexasHoldemPreferences {
                host_rules: PreviousTexasHoldemRuleSet {
                    player_count: TexasHoldemRuleSet::MAX_PLAYERS,
                    starting_chips: TexasHoldemRuleSet::default().starting_chips,
                    short_deck: true,
                },
            },
        },
    };
    let encoded = postcard::to_allocvec(&legacy).unwrap();
    let decoded = decode_preferences(&encoded).unwrap();

    assert_eq!(decoded.global.player_name, "旧版玩家");
    assert_eq!(decoded.games.qigui523.host_rules.deck_count, 3);
    assert!(decoded.games.texas_holdem.host_rules.short_deck);
    assert!(!decoded.games.texas_holdem.host_rules.ignore_kickers);
    assert!(!decoded.games.texas_holdem.host_rules.omaha);
    assert_eq!(decoded.games.shengji.host_rules, ShengjiRuleSet::default());
}

#[test]
fn remembered_host_rules_keep_valid_preferences_and_fixed_room_capacity() {
    let preferred = RuleSet {
        deck_count: 4,
        player_count: 2,
        hand_size: 15,
        time_control: TimeControl::Unlimited,
        advanced_play_types: true,
        ..RuleSet::default()
    };
    let normalized = normalize_host_rules(preferred);

    assert_eq!(normalized.player_count, TABLE_SEAT_COUNT);
    assert_eq!(normalized.deck_count, 4);
    assert_eq!(normalized.hand_size, 15);
    assert_eq!(normalized.time_control, TimeControl::Unlimited);
    assert!(normalized.advanced_play_types);
    assert!(normalized.validate().is_ok());
}

#[test]
fn saved_player_name_is_limited_by_unicode_characters() {
    let name = truncate_chars("一二三四五六七八", MAX_PLAYER_NAME_CHARS);
    assert_eq!(name, "一二三四五六七");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn ime_committed_chinese_is_accepted_by_the_player_name_filter() {
    let mut name = String::new();
    append_filtered_input(
        &mut name,
        InputField::PlayerName,
        "七鬼五二三玩家甲",
        MAX_PLAYER_NAME_CHARS,
    );

    assert_eq!(name, "七鬼五二三玩家");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn pasted_server_address_filters_whitespace_and_replaces_the_default_value() {
    let mut address = String::new();
    append_filtered_input(
        &mut address,
        InputField::JoinAddress,
        " 192.168.1.20:52300\n",
        64,
    );

    assert_eq!(address, "192.168.1.20:52300");

    let mut hostname = String::new();
    append_filtered_input(
        &mut hostname,
        InputField::JoinAddress,
        "frp-off.com:52436",
        64,
    );
    assert_eq!(hostname, "frp-off.com:52436");
}

#[test]
fn cards_are_displayed_from_high_to_low() {
    let mut cards = vec![
        Card::suited(0, Suit::Diamond, Rank::Four),
        Card::suited(0, Suit::Heart, Rank::Seven),
        Card::suited(0, Suit::Spade, Rank::Seven),
        Card::suited(0, Suit::Spade, Rank::Five),
    ];
    sort_cards_high_to_low(&mut cards);

    assert_eq!(cards[0], Card::suited(0, Suit::Spade, Rank::Seven));
    assert_eq!(cards[1], Card::suited(0, Suit::Heart, Rank::Seven));
    assert_eq!(cards[2], Card::suited(0, Suit::Spade, Rank::Five));
    assert_eq!(cards[3], Card::suited(0, Suit::Diamond, Rank::Four));
}

#[test]
fn greedy_hint_cycles_and_passes_when_no_response_exists() {
    let rules = RuleSet::default();
    let current_card = Card::suited(0, Suit::Diamond, Rank::Eight);
    let current = classify(&[current_card], &rules).unwrap();
    let hand = [
        Card::suited(0, Suit::Diamond, Rank::Nine),
        Card::suited(0, Suit::Diamond, Rank::Ten),
    ];
    let mut strategy = QiGui523Bot::new();

    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[0]])
    );
    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[1]])
    );
    assert_eq!(
        next_greedy_hint(&mut strategy, &hand, &current, &[current_card], &rules,),
        HintDecision::Select(vec![hand[0]])
    );

    let no_response = [Card::suited(0, Suit::Diamond, Rank::Six)];
    assert_eq!(
        next_greedy_hint(
            &mut strategy,
            &no_response,
            &current,
            &[current_card],
            &rules,
        ),
        HintDecision::Pass
    );
}

#[test]
fn turn_timer_switches_from_base_to_reserve_wording() {
    assert_eq!(
        turn_timer_label(Some(TurnTimerView {
            player: PlayerId(0),
            base_seconds: 5,
            reserve_seconds: 30,
        })),
        "5"
    );
    assert_eq!(
        turn_timer_label(Some(TurnTimerView {
            player: PlayerId(0),
            base_seconds: 0,
            reserve_seconds: 27,
        })),
        "烧条中... 27"
    );
}

#[test]
fn auto_playing_current_player_does_not_show_a_turn_clock() {
    let player = PlayerId(0);
    let mut game = leocard_protocol::QiGui523Snapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: player,
        host: player,
        players: vec![PlayerPublicState {
            id: player,
            profile_id: leocard_protocol::ProfileId([1; 32]),
            name: "玩家".to_owned(),
            avatar: None,
            seat: SeatId(0),
            hand_len: 1,
            score: 0,
            ready: false,
            connected: true,
            auto_play: false,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
        }],
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: leocard_protocol::StartingCardView {
            player,
            card: Card::suited(0, Suit::Diamond, Rank::Four),
        },
        trick: Some(leocard_protocol::TrickView {
            leader: player,
            current_player: player,
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
        }),
        turn_timer: Some(TurnTimerView {
            player,
            base_seconds: 5,
            reserve_seconds: 30,
        }),
        phase: GamePhaseView::Playing,
    };

    assert!(turn_clock_visible(&game, player));
    game.players[0].auto_play = true;
    assert!(!turn_clock_visible(&game, player));
}

#[test]
fn auto_play_overlay_is_a_full_width_cancel_button_above_the_hand_ui() {
    fn setup(mut commands: Commands, assets: Res<UiAssets>) {
        let parent = commands
            .spawn(Node {
                position_type: PositionType::Relative,
                ..default()
            })
            .id();
        add_auto_play_overlay(&mut commands, parent, &assets);
    }

    let mut app = App::new();
    app.init_resource::<UiAssets>();
    app.add_systems(Startup, setup);
    app.update();

    let mut query = app
        .world_mut()
        .query_filtered::<(&Node, &UiAction, &GlobalZIndex), With<AutoPlayOverlay>>();
    let (node, action, z_index) = query.single(app.world()).unwrap();
    assert!(matches!(action, UiAction::ToggleAutoPlay));
    assert_eq!(node.left, px(0));
    assert_eq!(node.right, px(0));
    assert_eq!(node.height, px(190));
    assert_eq!(*z_index, GlobalZIndex(1900));
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
    declaration: Option<leocard_protocol::ShengjiDeclarationView>,
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
                profile_id: leocard_protocol::ProfileId([id; 32]),
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
    game.declaration = Some(leocard_protocol::ShengjiDeclarationView {
        player: game.you,
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
        kind: leocard_shengji::BidKind::Initial,
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

    game.declaration = Some(leocard_protocol::ShengjiDeclarationView {
        player: game.you,
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
        kind: leocard_shengji::BidKind::Initial,
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
        Some(leocard_protocol::ShengjiDeclarationView {
            player: PlayerId(0),
            trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
            kind: leocard_shengji::BidKind::Initial,
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
        Some(leocard_protocol::ShengjiDeclarationView {
            player: PlayerId(1),
            trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
            kind: leocard_shengji::BidKind::Initial,
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

    game.declaration = Some(leocard_protocol::ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Diamond),
        kind: leocard_shengji::BidKind::Initial,
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

    game.declaration = Some(leocard_protocol::ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::NoTrumpSmallJoker,
        kind: leocard_shengji::BidKind::Counter,
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
        Some(leocard_protocol::ShengjiDeclarationView {
            player: PlayerId(1),
            trump: ShengjiBidTrump::NoTrumpBigJoker,
            kind: leocard_shengji::BidKind::Counter,
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
    let leocard_shengji::TrickPlay::Accepted(lead) =
        leocard_shengji::classify_lead(&lead_cards, trump, &ShengjiRuleSet::default(), &[])
            .unwrap()
    else {
        unreachable!("a single tractor is not a throw")
    };
    let mut game = shengji_ui_snapshot(hand, None);
    game.phase = ShengjiPhaseView::Playing;
    game.trump = Some(trump);
    game.current_player = Some(game.you);
    game.trick = Some(leocard_protocol::ShengjiTrickView {
        leader: PlayerId(1),
        current_player: game.you,
        winning_player: PlayerId(1),
        plays: vec![leocard_protocol::ShengjiPublicPlay {
            player: PlayerId(1),
            play: lead,
            throw_penalty: 0,
        }],
        table_points: 0,
    });
    let mut ui = UiState::default();

    select_forced_shengji_follow_cards(&game, &mut ui);

    assert_eq!(ui.selected_shengji, threes.into_iter().collect());

    let first = next_shengji_hint(&game, &ui.selected_shengji).unwrap();
    ui.selected_shengji = first.iter().copied().collect();
    let second = next_shengji_hint(&game, &ui.selected_shengji).unwrap();
    ui.selected_shengji = second.iter().copied().collect();
    let third = next_shengji_hint(&game, &ui.selected_shengji).unwrap();
    ui.selected_shengji = third.iter().copied().collect();
    let wrapped = next_shengji_hint(&game, &ui.selected_shengji).unwrap();

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
    let mut before = leocard_protocol::QiGui523Snapshot {
        match_id: MatchId([1; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        players: Vec::new(),
        your_hand: Vec::new(),
        draw_pile_len: 0,
        starting_card: leocard_protocol::StartingCardView {
            player: PlayerId(0),
            card: Card::suited(0, Suit::Diamond, Rank::Four),
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

    grace_after.declaration = Some(leocard_protocol::ShengjiDeclarationView {
        player: PlayerId(1),
        trump: ShengjiBidTrump::Suit(ShengjiSuit::Heart),
        kind: leocard_shengji::BidKind::Initial,
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
    app.insert_resource(ShengjiCardDragSelection::default());
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
    app.insert_resource(ShengjiCardDragSelection {
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
    assert!(ui.selected_shengji.contains(&card));
    assert!(ui.dirty);
}

#[test]
fn player_interaction_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(UiState::default());
    app.insert_resource(ChatPanelState::default());
    app.insert_resource(ConnectionForm::default());
    app.insert_resource(PlayerInteractionCooldown::default());
    app.insert_resource(ScoreCaptureEffectState::default());
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.add_systems(
        Update,
        (
            tick_player_interaction_cooldown,
            close_interaction_menu_on_outside_click,
            sync_opponent_badge_popups,
            sync_interaction_cooldown_masks,
            sync_chat_messages,
            sync_player_interactions,
            sync_score_capture_effect,
            animate_player_interactions,
            animate_score_capture_effects,
            animate_chat_bubbles,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn chat_bubbles_choose_the_inside_of_each_table_edge() {
    let layer = Vec2::new(1280.0, 720.0);
    let width = 200.0;
    let left = chat_bubble_position(Vec2::new(80.0, 360.0), layer, width);
    let right = chat_bubble_position(Vec2::new(1200.0, 360.0), layer, width);
    let top = chat_bubble_position(Vec2::new(640.0, 30.0), layer, width);

    assert!(left.x > 80.0);
    assert!(right.x + width < 1200.0);
    assert_eq!(top.x, 540.0);
    assert!(top.y >= 8.0);
}

#[test]
fn persistent_interaction_layer_is_not_owned_by_the_rebuilt_ui_root() {
    let mut app = App::new();
    app.add_systems(Startup, setup_camera);
    app.update();

    let layer = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerInteractionLayer>>()
        .single(app.world())
        .unwrap();
    assert!(app.world().get::<ChildOf>(layer).is_none());
}

#[test]
fn shengji_settlement_animation_systems_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(ShengjiSettlementAnimation::default());
    app.add_systems(
        Update,
        (
            animate_shengji_settlement_visuals,
            spawn_shengji_settlement_absorption,
            animate_shengji_score_absorbs,
        )
            .chain(),
    );
    app.update();
}

#[test]
fn button_feedback_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.add_systems(
        Update,
        (
            update_button_tints,
            play_button_click_sounds,
            animate_button_presses,
        )
            .chain(),
    );
    app.update();
}

#[test]
fn idle_button_animation_stops_writing_its_transform() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, animate_button_presses);
    let button = app
        .world_mut()
        .spawn((Button, Interaction::None, UiTransform::IDENTITY))
        .id();

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(
        !app.world()
            .entity(button)
            .get_ref::<UiTransform>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn settled_chat_panel_stops_writing_its_transform() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(ChatPanelState::default());
    app.add_systems(Update, animate_chat_panel);
    let panel = app
        .world_mut()
        .spawn((ChatPanel, UiTransform::IDENTITY))
        .id();

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    assert!(
        !app.world()
            .entity(panel)
            .get_ref::<UiTransform>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn interaction_cooldown_mask_reveals_itself_as_a_radial_sector() {
    let transparent = interaction_cooldown_mask_image(20, 20, 0.0);
    let half = interaction_cooldown_mask_image(20, 20, 0.5);
    let full = interaction_cooldown_mask_image(20, 20, 1.0);
    let covered = |image: &Image| {
        image
            .data
            .as_deref()
            .unwrap()
            .chunks_exact(4)
            .filter(|pixel| pixel[3] != 0)
            .count()
    };

    assert_eq!(covered(&transparent), 0);
    assert_eq!(covered(&full), 400);
    assert!((190..=210).contains(&covered(&half)));
}

#[test]
fn every_player_interaction_has_an_independent_cooldown() {
    let mut cooldown = PlayerInteractionCooldown::default();
    cooldown.start(PlayerInteractionKind::Shoe, 5.0);

    assert!(cooldown.is_active(PlayerInteractionKind::Shoe));
    assert!(!cooldown.is_active(PlayerInteractionKind::Flower));
    assert!(!cooldown.is_active(PlayerInteractionKind::Egg));
    assert!(!cooldown.is_active(PlayerInteractionKind::Wine));

    cooldown.start(PlayerInteractionKind::Egg, 0.5);
    cooldown.tick(0.5);
    assert!(!cooldown.is_active(PlayerInteractionKind::Egg));
    assert!(cooldown.is_active(PlayerInteractionKind::Shoe));
    assert!((cooldown.fraction(PlayerInteractionKind::Shoe) - 0.9).abs() < f32::EPSILON);
}

#[test]
fn summary_animation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(GameSummaryAnimation::default());
    app.add_systems(Update, animate_game_summary_visuals);

    app.update();
}

#[test]
fn time_control_options_follow_the_configured_order() {
    assert_eq!(previous_time_control(TimeControl::FivePlusTen), None);
    assert_eq!(
        next_time_control(TimeControl::FivePlusTen),
        Some(TimeControl::FivePlusThirty)
    );
    assert_eq!(
        next_time_control(TimeControl::FivePlusThirty),
        Some(TimeControl::FifteenPlusThirty)
    );
    assert_eq!(
        next_time_control(TimeControl::FifteenPlusThirty),
        Some(TimeControl::ThirtyPlusSixty)
    );
    assert_eq!(
        next_time_control(TimeControl::ThirtyPlusSixty),
        Some(TimeControl::Unlimited)
    );
    assert_eq!(
        previous_time_control(TimeControl::Unlimited),
        Some(TimeControl::ThirtyPlusSixty)
    );
    assert_eq!(next_time_control(TimeControl::Unlimited), None);
}

#[test]
fn play_error_toast_enters_upward_and_fades_out() {
    let mut toast = PlayErrorToast {
        active: true,
        entering: true,
        ..default()
    };
    let start = play_error_toast_visual(&toast);
    toast.elapsed = PLAY_ERROR_TOAST_ENTRY_DURATION;
    let entered = play_error_toast_visual(&toast);
    toast.elapsed = PLAY_ERROR_TOAST_DURATION;
    let finished = play_error_toast_visual(&toast);

    assert_eq!(start.opacity, 0.0);
    assert!(start.y > entered.y);
    assert!(entered.opacity > 0.99);
    assert_eq!(finished.opacity, 0.0);
}

#[test]
fn failed_throw_cards_reveal_split_rebound_and_return_to_the_player() {
    let direction = Vec2::new(0.0, 92.0);
    let stacked =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.0, direction);
    let revealed =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.24, direction);
    let split =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.54, direction);
    let rebounded =
        shengji_failed_throw_card_visual(ShengjiThrowFailureStage::Showing, 0, 5, 0.82, direction);
    let returned = shengji_failed_throw_card_visual(
        ShengjiThrowFailureStage::Returning,
        0,
        5,
        0.42,
        direction,
    );

    assert!(stacked.translation.x.abs() > revealed.translation.x.abs());
    assert!(split.translation.length() > revealed.translation.length());
    assert!(rebounded.translation.length() < split.translation.length());
    assert!(returned.translation.y > 80.0);
    assert!(!returned.visible);
}

#[test]
fn retriggered_play_error_toast_shakes_with_decay() {
    let mut toast = PlayErrorToast {
        active: true,
        shake_elapsed: Some(0.04),
        ..default()
    };
    let shaking = play_error_toast_visual(&toast);
    toast.shake_elapsed = Some(PLAY_ERROR_TOAST_SHAKE_DURATION);
    let settled = play_error_toast_visual(&toast);

    assert!(shaking.x.abs() > 1.0);
    assert_eq!(settled.x, 0.0);
}

#[test]
fn start_game_seats_smoothly_move_to_their_final_rectangles() {
    let start = start_game_seat_transition_visual(0.0);
    let moving = start_game_seat_transition_visual(START_GAME_SEAT_MOVE_DURATION * 0.5);
    let finished = start_game_seat_transition_visual(START_GAME_SEAT_MOVE_DURATION);

    assert_eq!(start.movement, 0.0);
    assert!(moving.movement > 0.0 && moving.movement < 1.0);
    assert!((moving.movement - 0.5).abs() < 0.00001);
    assert_eq!(finished.movement, 1.0);
}

#[test]
fn start_game_transition_can_be_attached_to_any_game_player_panel() {
    fn setup(mut commands: Commands) {
        let active = commands.spawn(Node::default()).id();
        attach_start_game_seat_transition(&mut commands, active, PlayerId(1), true);
        let settled = commands.spawn(Node::default()).id();
        attach_start_game_seat_transition(&mut commands, settled, PlayerId(2), false);
    }

    let mut app = App::new();
    app.add_systems(Startup, setup);
    app.update();

    let mut targets = app.world_mut().query::<(
        &GameSeatTransitionTarget,
        &GameSeatTransitionPose,
        &UiTransform,
        &Visibility,
    )>();
    let panels = targets
        .iter(app.world())
        .map(|(target, pose, transform, visibility)| {
            (target.0, (pose.initialized, *transform, *visibility))
        })
        .collect::<HashMap<_, _>>();
    assert_eq!(panels.len(), 2);
    assert_eq!(
        panels[&PlayerId(1)],
        (false, UiTransform::IDENTITY, Visibility::Hidden)
    );
    assert_eq!(
        panels[&PlayerId(2)],
        (false, UiTransform::IDENTITY, Visibility::Inherited)
    );
}

#[test]
fn start_game_seat_transition_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(StartGameSeatTransition::default());
    app.add_systems(Update, animate_start_game_seat_transition);

    app.update();
}

#[test]
fn turn_border_trace_eases_in_and_out_around_the_whole_perimeter() {
    let perimeter = 100.0;
    let start = turn_border_visible_interval(0.0, perimeter);
    let early = turn_border_visible_interval(0.095, perimeter);
    let halfway_grown = turn_border_visible_interval(0.475, perimeter);
    let full = turn_border_visible_interval(1.0, perimeter);
    let halfway_shrunk = turn_border_visible_interval(1.565, perimeter);
    let gap = turn_border_visible_interval(2.1, perimeter);

    assert_eq!(start, (0.0, 0.0));
    assert!(early.1 > 0.0 && early.1 < 10.0);
    assert!((halfway_grown.1 - 50.0).abs() < 0.001);
    assert_eq!(full, (0.0, perimeter));
    assert!((halfway_shrunk.0 - 50.0).abs() < 0.001);
    assert_eq!(halfway_shrunk.1, perimeter);
    assert_eq!(gap, (perimeter, perimeter));
}

#[test]
fn turn_border_animation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(Assets::<TurnBorderMaterial>::default());
    app.insert_resource(TurnBorderAnimationState::default());
    app.add_systems(Update, animate_turn_border_traces);

    app.update();
}

#[test]
fn shengji_presentation_system_queries_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(ShengjiPresentationState::default());
    app.add_systems(
        Update,
        (
            animate_shengji_presentation,
            animate_shengji_bottom_flip_markers,
            animate_shengji_power_outage_markers,
        ),
    );

    app.update();
}

#[test]
fn uno_interaction_and_presentation_systems_initialize_without_conflicts() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(UiState::default());
    app.insert_resource(UiAssets::default());
    app.insert_resource(UnoPresentationState::default());
    app.insert_resource(UnoAudioState::default());
    app.insert_resource(Assets::<UnoPaletteMaterial>::default());
    app.add_systems(
        Update,
        (
            animate_uno_hand_cards,
            spawn_uno_presentation_effects,
            animate_uno_flying_cards,
            sync_uno_discard_reveal,
            animate_uno_palette_effects,
            animate_uno_palette_selected_sectors,
            animate_uno_palette_color_rings,
            animate_uno_palette_particles,
            animate_uno_reverse_effects,
        )
            .chain(),
    );

    app.update();
}

#[test]
fn uno_presentation_waits_until_rebuilt_anchors_are_laid_out() {
    let node = ComputedNode::default();
    let transform = UiGlobalTransform::default();
    assert!(uno_anchor_in_layer(&node, &transform, &node, &transform).is_none());
}

#[test]
fn played_uno_card_settles_at_the_discard_cards_exact_scale() {
    assert_eq!(uno_flying_card_scale(false, 1.0), 1.0);
    assert!(uno_flying_card_scale(false, 0.5) < 1.0);
}

#[test]
fn jump_in_selection_allows_single_card_or_identical_pair() {
    let first = UnoCard::number(UnoColor::Red, 7, 0);
    let second = UnoCard::number(UnoColor::Red, 7, 1);
    let mut game = UnoSnapshot {
        match_id: MatchId([9; 16]),
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: UnoRuleSet {
            stack_skip: true,
            jump_in: true,
            ..UnoRuleSet::default()
        },
        players: vec![UnoPlayerState {
            id: PlayerId(0),
            profile_id: leocard_protocol::ProfileId([0; 32]),
            name: "玩家".to_owned(),
            avatar: None,
            seat: SeatId(0),
            hand_len: 3,
            ready: false,
            connected: true,
            auto_play: false,
            reference_points: 0,
            completed_games: 0,
            game_profiles: PlayerGameProfiles::default(),
            skipped_turns: 0,
        }],
        your_hand: vec![first, second, UnoCard::number(UnoColor::Blue, 3, 0)],
        draw_pile_len: 80,
        discard_top: UnoCard::number(UnoColor::Red, 4, 0),
        discard_pile: vec![UnoCard::number(UnoColor::Red, 4, 0)],
        current_color: Some(UnoColor::Red),
        current_player: Some(PlayerId(0)),
        direction: UnoDirection::Clockwise,
        pending_draw: 0,
        pending_kind: None,
        challenge_offender: None,
        pending_skip: 0,
        your_drawn_card: None,
        your_jump_in_card: None,
        uno_exposed: Vec::new(),
        uno_declared: Vec::new(),
        phase: UnoPhaseView::Playing,
    };

    let other = UnoCard::number(UnoColor::Blue, 3, 0);
    let mut selected = HashSet::new();
    toggle_uno_selection(Some(&game), &mut selected, first);
    assert_eq!(selected, HashSet::from([first]));
    toggle_uno_selection(Some(&game), &mut selected, second);
    assert_eq!(selected, HashSet::from([first, second]));
    toggle_uno_selection(Some(&game), &mut selected, first);
    assert_eq!(selected, HashSet::from([second]));
    toggle_uno_selection(Some(&game), &mut selected, other);
    assert_eq!(selected, HashSet::from([other]));

    game.uno_declared.push(game.you);
    assert_eq!(uno_pair_for_selection(&game, first), None);
}

#[test]
fn noninteractive_jump_in_card_uses_the_normal_selected_lift() {
    let card = UnoCard::number(UnoColor::Red, 7, 1);
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_millis(100));
    app.insert_resource(time);
    app.insert_resource(UiState {
        selected_uno: HashSet::from([card]),
        ..UiState::default()
    });
    app.add_systems(Update, animate_uno_hand_cards);
    let noninteractive_slot = app.world_mut().spawn_empty().id();
    let face = app
        .world_mut()
        .spawn((
            UnoHandCardVisual {
                button: noninteractive_slot,
                card,
                selected: true,
                hover_amount: 0.0,
                selected_amount: 0.0,
            },
            UiTransform::IDENTITY,
            Outline::default(),
            BoxShadow::new(Color::BLACK, px(0), px(0), px(0), px(0)),
            BorderColor::all(Color::NONE),
        ))
        .id();

    app.update();

    let visual = app.world().get::<UnoHandCardVisual>(face).unwrap();
    let transform = app.world().get::<UiTransform>(face).unwrap();
    assert!(visual.selected_amount > 0.0);
    assert!(matches!(transform.translation.y, Val::Px(y) if y < 0.0));
}

#[test]
fn authoritative_uno_discard_waits_for_its_flying_card_to_land() {
    let card = UnoCard::number(UnoColor::Red, 7, 0);
    let other = UnoCard::number(UnoColor::Blue, 7, 0);
    let mut presentation = UnoPresentationState::default();
    presentation.events.push_back(UnoEvent::CardPlayed {
        player: PlayerId(0),
        card,
        chosen_color: None,
        play_index: 0,
        play_count: 1,
    });

    assert!(uno_discard_should_be_hidden(
        card,
        &presentation,
        std::iter::empty()
    ));
    assert!(!uno_discard_should_be_hidden(
        other,
        &presentation,
        std::iter::empty()
    ));

    presentation.events.clear();
    assert!(uno_discard_should_be_hidden(
        card,
        &presentation,
        [Some(card)].into_iter()
    ));
    assert!(!uno_discard_should_be_hidden(
        card,
        &presentation,
        std::iter::empty()
    ));
}

#[test]
fn retained_uno_discard_cards_keep_their_pose_when_the_six_card_window_slides() {
    let cards = [
        UnoCard::number(UnoColor::Red, 1, 0),
        UnoCard::number(UnoColor::Yellow, 2, 0),
        UnoCard::number(UnoColor::Green, 3, 0),
        UnoCard::number(UnoColor::Blue, 4, 0),
        UnoCard::action(UnoColor::Red, UnoFace::Reverse, 0),
        UnoCard::action(UnoColor::Blue, UnoFace::Skip, 1),
        UnoCard::wild(UnoFace::Wild, 0),
    ];
    let before = cards[..6]
        .iter()
        .copied()
        .map(|card| (card, uno_discard_pose(card)))
        .collect::<HashMap<_, _>>();
    let after = cards[1..]
        .iter()
        .copied()
        .map(|card| (card, uno_discard_pose(card)))
        .collect::<HashMap<_, _>>();

    for card in &cards[1..6] {
        assert_eq!(before.get(card), after.get(card));
    }
}

#[test]
fn selected_uno_palette_sector_starts_at_base_size_then_grows() {
    assert_eq!(uno_palette_selected_scale(0.0), 1.0);
    assert_eq!(uno_palette_selected_scale(0.18), 1.0);
    assert!(uno_palette_selected_scale(0.70) >= 1.31);
    assert!(uno_palette_selected_scale(1.0) > 1.2);
}

#[test]
fn uno_reverse_effect_places_self_at_the_bottom_center_action_area() {
    let anchor = uno_reverse_own_anchor(Vec2::new(DESIGN_WIDTH, DESIGN_HEIGHT));
    assert_eq!(anchor, Vec2::new(640.0, 541.0));
}

#[test]
fn closed_chat_drawer_moves_its_border_fully_offscreen() {
    assert!(CHAT_PANEL_HIDDEN_OFFSET > CHAT_PANEL_WIDTH);
}

#[test]
fn closed_chat_drawer_translation_keeps_the_arrow_visible() {
    assert_eq!(
        chat_panel_translation(1.0),
        Val2::px(CHAT_PANEL_HIDDEN_OFFSET, 0.0)
    );
}

#[test]
fn held_raise_adjustment_stops_exactly_at_both_boundaries() {
    assert_eq!(texas_raise_repeat_value(6, -1, 4, 5, 20), 5);
    assert_eq!(texas_raise_repeat_value(19, 1, 4, 5, 20), 20);
    assert_eq!(texas_raise_repeat_value(12, -1, 3, 5, 20), 9);
    assert_eq!(texas_raise_repeat_value(12, 1, 3, 5, 20), 15);
}
