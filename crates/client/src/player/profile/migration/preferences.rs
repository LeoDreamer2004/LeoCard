use crate::player::preferences::{
    GamePreferences, GlobalPreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet, MahjongUmaStyle};
use leocard_protocol::PlayerGender;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
struct V032SavedPreferences {
    global: PreviousGlobalPreferences,
    games: V032GamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreviousSavedPreferences {
    global: PreviousGlobalPreferences,
    games: GamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreviousVoiceSavedPreferences {
    global: PreviousVoiceGlobalPreferences,
    games: GamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreviousTexasVoiceSavedPreferences {
    global: PreviousTexasVoiceGlobalPreferences,
    games: GamePreferences,
}

#[derive(Deserialize, Serialize)]
struct PreviousTexasVoiceGlobalPreferences {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    audio_volume: f32,
    gender: PlayerGender,
    mahjong_action_voices: bool,
    mahjong_fan_voices: bool,
}

impl From<PreviousTexasVoiceGlobalPreferences> for GlobalPreferences {
    fn from(previous: PreviousTexasVoiceGlobalPreferences) -> Self {
        Self {
            player_name: previous.player_name,
            avatar_png: previous.avatar_png,
            host_port: previous.host_port,
            join_address: previous.join_address,
            table_felt_path: previous.table_felt_path,
            table_brightness: previous.table_brightness,
            table_vignette: previous.table_vignette,
            audio_volume: previous.audio_volume,
            gender: previous.gender,
            mahjong_action_voices: previous.mahjong_action_voices,
            mahjong_fan_voices: previous.mahjong_fan_voices,
            texas_action_voices: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PreviousVoiceGlobalPreferences {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    audio_volume: f32,
    gender: PlayerGender,
}

impl From<PreviousVoiceGlobalPreferences> for GlobalPreferences {
    fn from(previous: PreviousVoiceGlobalPreferences) -> Self {
        Self {
            player_name: previous.player_name,
            avatar_png: previous.avatar_png,
            host_port: previous.host_port,
            join_address: previous.join_address,
            table_felt_path: previous.table_felt_path,
            table_brightness: previous.table_brightness,
            table_vignette: previous.table_vignette,
            audio_volume: previous.audio_volume,
            gender: previous.gender,
            mahjong_action_voices: true,
            mahjong_fan_voices: true,
            texas_action_voices: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct PreviousGlobalPreferences {
    player_name: String,
    avatar_png: Option<Vec<u8>>,
    host_port: String,
    join_address: String,
    table_felt_path: Option<PathBuf>,
    table_brightness: f32,
    table_vignette: f32,
    #[serde(default = "previous_audio_volume")]
    audio_volume: f32,
}

fn previous_audio_volume() -> f32 {
    0.8
}

impl From<PreviousGlobalPreferences> for GlobalPreferences {
    fn from(previous: PreviousGlobalPreferences) -> Self {
        Self {
            player_name: previous.player_name,
            gender: PlayerGender::Male,
            avatar_png: previous.avatar_png,
            host_port: previous.host_port,
            join_address: previous.join_address,
            table_felt_path: previous.table_felt_path,
            table_brightness: previous.table_brightness,
            table_vignette: previous.table_vignette,
            audio_volume: previous.audio_volume,
            mahjong_action_voices: true,
            mahjong_fan_voices: true,
            texas_action_voices: true,
        }
    }
}

#[derive(Deserialize, Serialize)]
struct V032GamePreferences {
    qigui523: QiGui523Preferences,
    texas_holdem: TexasHoldemPreferences,
    shengji: ShengjiPreferences,
    uno: UnoPreferences,
    mahjong: V032MahjongPreferences,
}

#[derive(Deserialize, Serialize)]
struct V032MahjongPreferences {
    host_rules: V032MahjongRuleSet,
}

#[derive(Deserialize, Serialize)]
struct V032MahjongRuleSet {
    match_length: MahjongMatchLength,
    minimum_eight_points: bool,
    multiple_winners: bool,
    false_win: bool,
}

pub(in crate::player) fn decode_player_preferences(bytes: &[u8]) -> Option<SavedPreferences> {
    postcard::from_bytes(bytes)
        .ok()
        .or_else(|| {
            let previous: PreviousTexasVoiceSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global.into(),
                games: previous.games,
            })
        })
        .or_else(|| {
            let previous: PreviousVoiceSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global.into(),
                games: previous.games,
            })
        })
        .or_else(|| {
            let previous: PreviousSavedPreferences = postcard::from_bytes(bytes).ok()?;
            Some(SavedPreferences {
                global: previous.global.into(),
                games: previous.games,
            })
        })
        .or_else(|| {
            let previous: V032SavedPreferences = postcard::from_bytes(bytes).ok()?;
            let old_rules = previous.games.mahjong.host_rules;
            Some(SavedPreferences {
                global: previous.global.into(),
                games: GamePreferences {
                    qigui523: previous.games.qigui523,
                    texas_holdem: previous.games.texas_holdem,
                    shengji: previous.games.shengji,
                    uno: previous.games.uno,
                    mahjong: MahjongPreferences {
                        host_rules: MahjongRuleSet {
                            match_length: old_rules.match_length,
                            uma_style: MahjongUmaStyle::Balanced,
                            minimum_eight_points: old_rules.minimum_eight_points,
                            multiple_winners: old_rules.multiple_winners,
                            false_win: old_rules.false_win,
                        },
                    },
                },
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_without_gender_keep_the_name_and_default_to_male() {
        let previous = PreviousSavedPreferences {
            global: PreviousGlobalPreferences {
                player_name: "旧玩家".to_owned(),
                avatar_png: None,
                host_port: "52300".to_owned(),
                join_address: "127.0.0.1:52300".to_owned(),
                table_felt_path: None,
                table_brightness: 0.9,
                table_vignette: 0.4,
                audio_volume: 0.7,
            },
            games: SavedPreferences::default().games,
        };
        let encoded = postcard::to_allocvec(&previous).unwrap();
        let decoded = decode_player_preferences(&encoded).unwrap();
        assert_eq!(decoded.global.player_name, "旧玩家");
        assert_eq!(decoded.global.gender, PlayerGender::Male);
        assert_eq!(decoded.global.audio_volume, 0.7);
    }

    #[test]
    fn preferences_without_voice_switches_keep_existing_settings() {
        let previous = PreviousVoiceSavedPreferences {
            global: PreviousVoiceGlobalPreferences {
                player_name: "旧玩家".to_owned(),
                avatar_png: None,
                host_port: "52300".to_owned(),
                join_address: "127.0.0.1:52300".to_owned(),
                table_felt_path: None,
                table_brightness: 0.9,
                table_vignette: 0.4,
                audio_volume: 0.7,
                gender: PlayerGender::Female,
            },
            games: SavedPreferences::default().games,
        };
        let encoded = postcard::to_allocvec(&previous).unwrap();
        let decoded = decode_player_preferences(&encoded).unwrap();
        assert_eq!(decoded.global.gender, PlayerGender::Female);
        assert_eq!(decoded.global.audio_volume, 0.7);
        assert!(decoded.global.mahjong_action_voices);
        assert!(decoded.global.mahjong_fan_voices);
    }
}
