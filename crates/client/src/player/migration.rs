//! 旧版本地档案与偏好的读取迁移。

use super::super::preferences::{
    GamePreferences, GlobalPreferences, MahjongPreferences, QiGui523Preferences, SavedPreferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
use super::{StoredGameProfiles, StoredPlayerProfile, StoredRatingProfile};
use leocard_mahjong::{MahjongMatchLength, MahjongRuleSet, MahjongUmaStyle};
use leocard_protocol::{
    MatchId, PlayerInteractionStats, ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};
use serde::{Deserialize, Serialize};

macro_rules! stored_profile_compatibility_types {
    ($(($profile:ident, $games:ident, {$($field:ident: $ty:ty),* $(,)?})),* $(,)?) => {
        $(
            #[derive(Deserialize, Serialize)]
            struct $profile {
                secret_key: [u8; 32],
                games: $games,
            }

            #[derive(Deserialize, Serialize)]
            struct $games {
                $($field: $ty,)*
            }
        )*
    };
}

stored_profile_compatibility_types!(
    // 0.3.2：已有互动统计，尚无麻将统计。
    (V032StoredPlayerProfile, V032StoredGameProfiles, {
        qigui523: StoredRatingProfile,
        texas_holdem_stats: Option<TexasHoldemProfileStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<UnoProfileStats>,
        interaction_stats: Option<PlayerInteractionStats>,
    }),
    (PreInteractionStoredPlayerProfile, PreInteractionStoredGameProfiles, {
        qigui523: StoredRatingProfile,
        texas_holdem_stats: Option<TexasHoldemProfileStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<UnoProfileStats>,
    }),
    (PreUnoStoredPlayerProfile, PreUnoStoredGameProfiles, {
        qigui523: StoredRatingProfile,
        texas_holdem_stats: Option<TexasHoldemProfileStats>,
        shengji_stats: Option<ShengjiProfileStats>,
    }),
    (PreShengjiStoredPlayerProfile, PreShengjiStoredGameProfiles, {
        qigui523: StoredRatingProfile,
        texas_holdem_stats: Option<TexasHoldemProfileStats>,
    }),
    (PreTexasStoredPlayerProfile, PreTexasStoredGameProfiles, {
        qigui523: StoredRatingProfile,
    }),
    (PreDetailedStoredPlayerProfile, PreDetailedStoredGameProfiles, {
        qigui523: PreDetailedStoredRatingProfile,
    }),
);

#[derive(Deserialize, Serialize)]
struct PreDetailedStoredRatingProfile {
    reference_points: i32,
    completed_games: u32,
    applied_matches: Vec<MatchId>,
}

#[derive(Deserialize, Serialize)]
struct V032SavedPreferences {
    global: GlobalPreferences,
    games: V032GamePreferences,
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
    postcard::from_bytes(bytes).ok().or_else(|| {
        let previous: V032SavedPreferences = postcard::from_bytes(bytes).ok()?;
        let old_rules = previous.games.mahjong.host_rules;
        Some(SavedPreferences {
            global: previous.global,
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

pub(super) fn decode_player_profile(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    match postcard::from_bytes(bytes) {
        Ok(stored) => Ok(stored),
        Err(current_error) => {
            if let Ok(previous) = postcard::from_bytes::<V032StoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats,
                        interaction_stats: previous.games.interaction_stats,
                        mahjong_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreInteractionStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats,
                        interaction_stats: None,
                        mahjong_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreUnoStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreShengjiStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                    },
                });
            }
            if let Ok(previous) = postcard::from_bytes::<PreTexasStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: None,
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                    },
                });
            }
            let previous: PreDetailedStoredPlayerProfile = postcard::from_bytes(bytes)
                .map_err(|_| format!("玩家档案已损坏：{current_error}"))?;
            Ok(StoredPlayerProfile {
                secret_key: previous.secret_key,
                games: StoredGameProfiles {
                    qigui523: StoredRatingProfile {
                        reference_points: previous.games.qigui523.reference_points,
                        completed_games: previous.games.qigui523.completed_games,
                        applied_matches: previous.games.qigui523.applied_matches,
                        qigui523_stats: None,
                    },
                    texas_holdem_stats: None,
                    shengji_stats: None,
                    uno_stats: None,
                    interaction_stats: None,
                    mahjong_stats: None,
                },
            })
        }
    }
}
