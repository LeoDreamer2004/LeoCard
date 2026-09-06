use super::*;
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet};
use leocard_protocol::TABLE_SEAT_COUNT;
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
        },
        games: GamePreferences {
            qigui523: QiGui523Preferences {
                host_rules: QiGuiRuleSet {
                    deck_count: 3,
                    hand_size: 12,
                    time_control: TimeControl::ThirtyPlusSixty,
                    advanced_play_types: true,
                    ..normalize_host_rules(QiGuiRuleSet::default())
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
    let normalized = normalize_host_rules(preferred);

    assert_eq!(normalized.player_count, TABLE_SEAT_COUNT);
    assert_eq!(normalized.deck_count, 4);
    assert_eq!(normalized.hand_size, 15);
    assert_eq!(normalized.time_control, TimeControl::Unlimited);
    assert!(normalized.advanced_play_types);
    assert!(normalized.validate().is_ok());
}
