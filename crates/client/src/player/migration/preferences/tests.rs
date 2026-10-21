use super::schema::{Games, Rules, TexasRules};
use crate::player::preferences::{
    GamePreferences, PREFERENCES_HEADER, SavedPreferences, decode_player_preferences,
};
use leocard_minecraft_guess::{GuessMode, GuessPool, MinecraftGuessRuleSet};
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

#[test]
fn version_one_keeps_all_existing_settings_and_defaults_unpublished_minecraft_rules() {
    let mut saved = SavedPreferences::default();
    saved.global.player_name = "原有偏好".to_owned();
    saved.global.avatar_png = Some(vec![1, 2, 3]);
    saved.global.gender = PlayerGender::Female;
    saved.global.audio_volume = 0.35;
    saved.global.mahjong_action_voices = false;
    saved.games.qigui523.host_rules.deck_count = 4;
    saved.games.texas_holdem.host_rules.fixed_limit = true;
    saved.games.texas_holdem.host_rules.starting_chips = 40;
    saved.games.shengji.host_rules.deck_count = 4;
    saved.games.uno.host_rules.uno_callout = false;
    saved.games.mahjong.host_rules.multiple_winners = true;
    let global = postcard::to_allocvec(&saved.global).unwrap();
    // Freeze the five-game layout independently of the new GamePreferences structure.
    let games = postcard::to_allocvec(&(
        saved.games.qigui523,
        saved.games.texas_holdem,
        saved.games.shengji,
        saved.games.uno,
        saved.games.mahjong,
    ))
    .unwrap();
    let mut bytes = b"LeoCard/prefs/v1\0".to_vec();
    bytes.extend(&global);
    bytes.extend(&games);
    let migrated = decode_player_preferences(&bytes).unwrap();
    assert_eq!(postcard::to_allocvec(&migrated.global).unwrap(), global);
    assert_eq!(
        migrated.games.minecraft_guess.host_rules,
        MinecraftGuessRuleSet::default()
    );
    assert_eq!(
        postcard::to_allocvec(&(
            migrated.games.qigui523,
            migrated.games.texas_holdem,
            migrated.games.shengji,
            migrated.games.uno,
            migrated.games.mahjong,
        ))
        .unwrap(),
        games
    );
}

#[test]
fn current_preferences_remember_all_minecraft_modes_in_the_shared_archive() {
    let mut saved = SavedPreferences::default();
    let rules = &mut saved.games.minecraft_guess.host_rules;
    rules.mode = GuessMode::Graph;
    rules.turn_based.rounds = 4;
    rules.turn_based.options.pool = GuessPool::Items;
    rules.race.games = 10;
    rules.race.seconds = 180;
    rules.race.options.candidate_hints = false;
    rules.race.options.allow_unknown_guesses = true;
    rules.graph.nodes = 32;
    rules.graph.round_robin = false;
    rules.graph.temperature = 9;
    rules.graph.guess_seconds = 240;
    rules.graph.include_tags = false;
    rules.graph.show_character_count = false;
    rules.graph.show_directions = true;
    assert!(rules.is_valid());
    let mut bytes = PREFERENCES_HEADER.to_vec();
    bytes.extend(postcard::to_allocvec(&saved).unwrap());
    let decoded = decode_player_preferences(&bytes).unwrap();
    assert_eq!(
        decoded.games.minecraft_guess.host_rules,
        saved.games.minecraft_guess.host_rules
    );
}

#[test]
fn version_two_keeps_minecraft_settings_and_adds_the_original_round_robin_format() {
    let mut saved = SavedPreferences::default();
    saved.global.player_name = "已发布玩家".to_owned();
    saved.games.texas_holdem.host_rules.fixed_limit = true;
    let mut rules = MinecraftGuessRuleSet {
        mode: GuessMode::Graph,
        ..Default::default()
    };
    rules.turn_based.rounds = 3;
    rules.race.seconds = 90;
    rules.graph.nodes = 24;
    rules.graph.temperature = 7;
    rules.graph.include_tags = false;
    let graph = rules.graph;
    let mut bytes = b"LeoCard/prefs/v2\0".to_vec();
    bytes.extend(
        postcard::to_allocvec(&(
            &saved.global,
            (
                saved.games.qigui523,
                saved.games.texas_holdem,
                saved.games.shengji,
                saved.games.uno,
                saved.games.mahjong,
            ),
            (
                rules.mode,
                rules.turn_based,
                rules.race,
                (
                    graph.options,
                    graph.nodes,
                    graph.temperature,
                    graph.guess_seconds,
                    graph.show_character_count,
                    graph.show_directions,
                    graph.include_tags,
                ),
            ),
        ))
        .unwrap(),
    );
    let decoded = decode_player_preferences(&bytes).unwrap();
    assert_eq!(decoded.global.player_name, saved.global.player_name);
    assert!(decoded.games.texas_holdem.host_rules.fixed_limit);
    assert_eq!(decoded.games.minecraft_guess.host_rules, rules);
    bytes.push(42);
    assert!(decode_player_preferences(&bytes).is_none());
}
