use super::prelude::*;
use leocard_client::GlobalPreferences;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::{PlayerGender, TABLE_SEAT_COUNT};
use leocard_qigui523::{QiGuiRuleSet, TimeControl};
use leocard_shengji::ShengjiRuleSet;
use leocard_texas_holdem::TexasHoldemRuleSet;
use leocard_uno::UnoRuleSet;
use std::path::PathBuf;

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
            gender: PlayerGender::Female,
            mahjong_action_voices: false,
            mahjong_fan_voices: true,
            texas_action_voices: false,
        },
        games: GamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: QiGuiRuleSet {
                    deck_count: 3,
                    hand_size: 12,
                    time_control: TimeControl::ThirtyPlusSixty,
                    advanced_play_types: true,
                    ..normalize_rules(QiGuiRuleSet::default())
                },
            },
            texas_holdem: TexasHoldemPreferences {
                host_rules: TexasHoldemRuleSet {
                    starting_chips: 40,
                    squid_markers: 2,
                    deepseek_squid_double: true,
                    short_deck: true,
                    ignore_kickers: true,
                    omaha: true,
                    fixed_limit: true,
                    ..normalize_rules(TexasHoldemRuleSet::default())
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
                ..ShengjiPreferences::default()
            },
            uno: UnoPreferences {
                host_rules: UnoRuleSet {
                    action_stacking: true,
                    uno_callout: false,
                    skip_draw_penalty: true,
                    jump_in: true,
                    swap_pack: true,
                    reverse_pack: true,
                    stack_pack: true,
                    ..UnoRuleSet::default()
                },
            },
            mahjong: MahjongPreferences {
                host_rules: MahjongRuleSet {
                    match_length: MahjongMatchLength::FullGame,
                    minimum_eight_points: false,
                    multiple_winners: true,
                    false_win: false,
                    ..MahjongRuleSet::default()
                },
            },
            minecraft_guess: Default::default(),
            doudizhu: Default::default(),
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
    assert_eq!(decoded.global.gender, PlayerGender::Female);
    assert!(!decoded.global.mahjong_action_voices);
    assert!(decoded.global.mahjong_fan_voices);
    assert_eq!(
        decoded.games.qigui523.host_rules,
        saved.games.qigui523.host_rules
    );
    assert_eq!(
        decoded.games.shengji.host_rules,
        saved.games.shengji.host_rules
    );
    assert_eq!(decoded.games.uno.host_rules, saved.games.uno.host_rules);
    assert_eq!(
        decoded.games.mahjong.host_rules,
        saved.games.mahjong.host_rules
    );
}

#[test]
fn remembered_host_rules_keep_valid_preferences_and_fixed_room_capacity() {
    let preferred = QiGuiRuleSet {
        deck_count: 4,
        player_count: 2,
        hand_size: 15,
        time_control: TimeControl::Unlimited,
        advanced_play_types: true,
        ..QiGuiRuleSet::default()
    };
    let normalized = normalize_rules(preferred);

    assert_eq!(normalized.player_count, TABLE_SEAT_COUNT);
    assert_eq!(normalized.deck_count, 4);
    assert_eq!(normalized.hand_size, 15);
    assert_eq!(normalized.time_control, TimeControl::Unlimited);
    assert!(normalized.advanced_play_types);
    assert!(normalized.validate().is_ok());
}

#[test]
fn shengji_modes_keep_separate_rule_preferences() {
    use leocard_shengji::ShengjiMode;
    let classic = ShengjiRuleSet {
        deck_count: 4,
        five_trump_crossing: true,
        ..ShengjiRuleSet::default()
    };
    let six = ShengjiRuleSet {
        bottom_copy: true,
        constant_trump: true,
        ..ShengjiRuleSet::for_mode(ShengjiMode::SixPlayer)
    };
    let preferences = ShengjiPreferences {
        host_rules: six,
        mode_rules: [classic, six],
    };
    let saved = serde_json::to_vec(&preferences).unwrap();
    let loaded: ShengjiPreferences = serde_json::from_slice(&saved).unwrap();
    let remembered = loaded.remembered_modes();
    assert_eq!(remembered, [classic, six]);
    assert_eq!(loaded.host_rules.mode, ShengjiMode::SixPlayer);
    assert!(remembered.iter().all(|rules| rules.validate().is_ok()));
    assert_eq!(remembered[0].deck_count, 4);
    assert!(remembered[0].five_trump_crossing);
    assert_eq!(remembered[1].deck_count, 3);
    assert!(!remembered[1].five_trump_crossing);
}
