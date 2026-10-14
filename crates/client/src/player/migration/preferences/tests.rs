use super::schema::{Games, Rules, TexasRules};
use crate::player::preferences::{
    GamePreferences, PREFERENCES_HEADER, SavedPreferences, decode_player_preferences,
};
use leocard_protocol::PlayerGender;
use std::path::PathBuf;

// 使用旧存档的平铺字段独立生成字节，验证组合结构不会改变二进制布局。
fn archive_bytes(extra_global_fields: &[u8], games: GamePreferences) -> Vec<u8> {
    let mut bytes = postcard::to_allocvec(&(
        "旧玩家",
        None::<Vec<u8>>,
        "52300",
        "127.0.0.1:52300",
        None::<PathBuf>,
        0.9_f32,
        0.4_f32,
        0.7_f32,
    ))
    .unwrap();
    bytes.extend(extra_global_fields);
    bytes.extend(postcard::to_allocvec(&legacy_games(games)).unwrap());
    bytes
}

fn legacy_games(games: GamePreferences) -> Games {
    let rules = games.texas_holdem.host_rules;
    Games {
        qigui523: games.qigui523,
        texas_holdem: Rules {
            host_rules: TexasRules {
                player_count: rules.player_count,
                starting_chips: rules.starting_chips,
                short_deck: rules.short_deck,
                ignore_kickers: rules.ignore_kickers,
                omaha: rules.omaha,
            },
        },
        shengji: games.shengji,
        uno: games.uno,
        mahjong: games.mahjong,
    }
}

#[test]
fn preferences_without_gender_keep_the_name_and_default_to_male() {
    let bytes = archive_bytes(&[], SavedPreferences::default().games);
    let decoded = decode_player_preferences(&bytes).unwrap();
    assert_eq!(decoded.global.player_name, "旧玩家");
    assert_eq!(decoded.global.gender, PlayerGender::Male);
    assert_eq!(decoded.global.audio_volume, 0.7);
}

#[test]
fn preferences_without_voice_switches_keep_existing_settings() {
    let gender = postcard::to_allocvec(&PlayerGender::Female).unwrap();
    let bytes = archive_bytes(&gender, SavedPreferences::default().games);
    let decoded = decode_player_preferences(&bytes).unwrap();
    assert_eq!(decoded.global.gender, PlayerGender::Female);
    assert_eq!(decoded.global.audio_volume, 0.7);
    assert!(decoded.global.mahjong_action_voices);
    assert!(decoded.global.mahjong_fan_voices);
}

#[test]
fn betting_preferences_keep_rules_and_current_settings_round_trip() {
    let mut saved = SavedPreferences::default();
    saved.games.texas_holdem.host_rules.starting_chips = 40;
    saved.games.texas_holdem.host_rules.omaha = true;
    saved.games.texas_holdem.host_rules.short_deck = true;
    saved.games.shengji.host_rules.deck_count = 4;
    let voice_settings =
        postcard::to_allocvec(&(PlayerGender::Female, false, true, false)).unwrap();
    let mut decoded =
        decode_player_preferences(&archive_bytes(&voice_settings, saved.games)).unwrap();
    assert_eq!(decoded.global.player_name, "旧玩家");
    assert_eq!(decoded.global.audio_volume, 0.7);
    assert_eq!(decoded.global.gender, PlayerGender::Female);
    assert!(!decoded.global.mahjong_action_voices);
    assert!(decoded.global.mahjong_fan_voices);
    assert!(!decoded.global.texas_action_voices);
    assert_eq!(decoded.games.texas_holdem.host_rules.starting_chips, 40);
    assert!(decoded.games.texas_holdem.host_rules.omaha);
    assert!(decoded.games.texas_holdem.host_rules.short_deck);
    assert!(!decoded.games.texas_holdem.host_rules.fixed_limit);
    assert_eq!(decoded.games.shengji.host_rules.deck_count, 4);
    decoded.games.texas_holdem.host_rules.fixed_limit = true;
    let mut encoded = PREFERENCES_HEADER.to_vec();
    encoded.extend(postcard::to_allocvec(&decoded).unwrap());
    let current = decode_player_preferences(&encoded).unwrap();
    assert_eq!(
        current.games.texas_holdem.host_rules,
        decoded.games.texas_holdem.host_rules
    );
    assert_eq!(current.global.player_name, "旧玩家");
}
