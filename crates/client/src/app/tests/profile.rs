use super::*;
use leocard_client::PlayerIdentity;

use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerId, PlayerInteractionKind, PlayerInteractionStats,
    PlayerReferenceChange, QiGui523ProfileStats, ShengjiProfileStats, TexasHoldemProfileStats,
    UnoProfileStats,
};
use std::collections::HashSet;

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
    let changes = [PlayerReferenceChange {
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
            PlayerMenuProfile {
                name: "远端玩家",
                avatar: None,
                reference_points: 500,
                completed_games: 37,
                game_profiles: &PlayerGameProfiles::default(),
            },
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
            .social
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
