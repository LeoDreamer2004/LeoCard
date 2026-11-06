use super::super::{PROFILE_HEADER, StoredPlayerProfile};
use super::archive::{LegacyRatingProfile, V6_PROFILE_HEADER, V6GameProfiles, V6PlayerProfile};
use super::legacy::{MahjongProfileStats, ShengjiProfileStats};
use super::mc_guess::{PreviousMcGuessStats, SixSeatMcGuessStats};
use super::texas::V3TexasStats;
use super::uno::SixSeatUnoStats;
use leocard_protocol::{MatchId, PlayerInteractionStats};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

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
    (SixSeatUnoStoredPlayerProfile, SixSeatUnoStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<SixSeatUnoStats>,
        interaction_stats: Option<PlayerInteractionStats>,
        mahjong_stats: Option<MahjongProfileStats>,
        mc_guess_stats: Option<PreviousMcGuessStats>,
    }),
    (SixSeatStoredPlayerProfile, SixSeatStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<SixSeatUnoStats>,
        interaction_stats: Option<PlayerInteractionStats>,
        mahjong_stats: Option<MahjongProfileStats>,
        mc_guess_stats: Option<SixSeatMcGuessStats>,
    }),
    // Through 0.8.0: MC guessing contributed only to the overall rating.
    (V080StoredPlayerProfile, V080StoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<SixSeatUnoStats>,
        interaction_stats: Option<PlayerInteractionStats>,
        mahjong_stats: Option<MahjongProfileStats>,
    }),
    // 0.3.2：已有互动统计，尚无麻将统计。
    (V032StoredPlayerProfile, V032StoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<SixSeatUnoStats>,
        interaction_stats: Option<PlayerInteractionStats>,
    }),
    (PreInteractionStoredPlayerProfile, PreInteractionStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
        uno_stats: Option<SixSeatUnoStats>,
    }),
    (PreUnoStoredPlayerProfile, PreUnoStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
        shengji_stats: Option<ShengjiProfileStats>,
    }),
    (PreShengjiStoredPlayerProfile, PreShengjiStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
        texas_stats: Option<V3TexasStats>,
    }),
    (PreTexasStoredPlayerProfile, PreTexasStoredGameProfiles, {
        qigui523: LegacyRatingProfile,
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

pub(super) fn decode_exact<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, postcard::Error> {
    let (value, rest) = postcard::take_from_bytes(bytes)?;
    if !rest.is_empty() {
        return Err(postcard::Error::DeserializeBadEncoding);
    }
    Ok(value)
}

pub(in crate::player::profile) fn decode_player_profile(
    bytes: &[u8],
) -> Result<StoredPlayerProfile, String> {
    // A schema marker prevents a six-placement archive from being interpreted
    // as eight placements merely because the following counters happen to fit.
    if let Some(current) = bytes.strip_prefix(PROFILE_HEADER) {
        return decode_exact(current).map_err(|error| format!("玩家档案已损坏：{error}"));
    }
    if let Some(previous) = bytes.strip_prefix(super::v8::V8_PROFILE_HEADER) {
        return super::v8::decode_v8(previous);
    }
    if let Some(previous) = bytes.strip_prefix(super::guandan::V7_PROFILE_HEADER) {
        return super::guandan::decode_v7(previous);
    }
    decode_legacy_profile(bytes).map(Into::into)
}

fn decode_legacy_profile(bytes: &[u8]) -> Result<V6PlayerProfile, String> {
    if let Some(previous) = bytes.strip_prefix(V6_PROFILE_HEADER) {
        return decode_exact(previous).map_err(|error| format!("玩家档案已损坏：{error}"));
    }
    if let Some(previous) = bytes.strip_prefix(super::doudizhu::V5_PROFILE_HEADER) {
        return super::doudizhu::decode_v5(previous);
    }
    if let Some(previous) = bytes.strip_prefix(super::texas::V4_PROFILE_HEADER) {
        return super::texas::decode_v4(previous);
    }
    if let Some(previous) = bytes.strip_prefix(super::texas::V3_PROFILE_HEADER) {
        return super::texas::decode_v3(previous);
    }
    if let Some(previous) = bytes.strip_prefix(super::graph::V2_PROFILE_HEADER) {
        return super::graph::decode_v2(previous);
    }
    if let Some(previous) = bytes.strip_prefix(super::blackjack::V1_PROFILE_HEADER) {
        return super::blackjack::decode_v1(previous);
    }
    if bytes.starts_with(b"LeoCard/profile/") {
        return Err("不支持此玩家档案版本，原文件已保留".to_owned());
    }
    match decode_exact::<SixSeatUnoStoredPlayerProfile>(bytes) {
        Ok(previous) => Ok(V6PlayerProfile {
            secret_key: previous.secret_key,
            games: V6GameProfiles {
                doudizhu_stats: None,
                blackjack_stats: None,
                qigui523: previous.games.qigui523,
                texas_stats: previous.games.texas_stats.map(Into::into),
                shengji_stats: previous.games.shengji_stats,
                uno_stats: previous.games.uno_stats.map(Into::into),
                interaction_stats: previous.games.interaction_stats,
                mahjong_stats: previous.games.mahjong_stats,
                mc_guess_stats: previous.games.mc_guess_stats.map(Into::into),
            },
        }),
        Err(current_error) => {
            if let Ok(previous) = decode_exact::<SixSeatStoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats.map(Into::into),
                        interaction_stats: previous.games.interaction_stats,
                        mahjong_stats: previous.games.mahjong_stats,
                        mc_guess_stats: previous.games.mc_guess_stats.map(Into::into),
                    },
                });
            }
            if let Ok(previous) = decode_exact::<V080StoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats.map(Into::into),
                        interaction_stats: previous.games.interaction_stats,
                        mahjong_stats: previous.games.mahjong_stats,
                        mc_guess_stats: None,
                    },
                });
            }
            if let Ok(previous) = decode_exact::<V032StoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats.map(Into::into),
                        interaction_stats: previous.games.interaction_stats,
                        mahjong_stats: None,
                        mc_guess_stats: None,
                    },
                });
            }
            if let Ok(previous) = decode_exact::<PreInteractionStoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats.map(Into::into),
                        interaction_stats: None,
                        mahjong_stats: None,
                        mc_guess_stats: None,
                    },
                });
            }
            if let Ok(previous) = decode_exact::<PreUnoStoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                        mc_guess_stats: None,
                    },
                });
            }
            if let Ok(previous) = decode_exact::<PreShengjiStoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: previous.games.texas_stats.map(Into::into),
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                        mc_guess_stats: None,
                    },
                });
            }
            if let Ok(previous) = decode_exact::<PreTexasStoredPlayerProfile>(bytes) {
                return Ok(V6PlayerProfile {
                    secret_key: previous.secret_key,
                    games: V6GameProfiles {
                        doudizhu_stats: None,
                        blackjack_stats: None,
                        qigui523: previous.games.qigui523,
                        texas_stats: None,
                        shengji_stats: None,
                        uno_stats: None,
                        interaction_stats: None,
                        mahjong_stats: None,
                        mc_guess_stats: None,
                    },
                });
            }
            let previous: PreDetailedStoredPlayerProfile =
                decode_exact(bytes).map_err(|_| format!("玩家档案已损坏：{current_error}"))?;
            Ok(V6PlayerProfile {
                secret_key: previous.secret_key,
                games: V6GameProfiles {
                    doudizhu_stats: None,
                    blackjack_stats: None,
                    qigui523: LegacyRatingProfile {
                        reference_points: previous.games.qigui523.reference_points,
                        completed_games: previous.games.qigui523.completed_games,
                        applied_matches: previous.games.qigui523.applied_matches,
                        qigui523_stats: None,
                    },
                    texas_stats: None,
                    shengji_stats: None,
                    uno_stats: None,
                    interaction_stats: None,
                    mahjong_stats: None,
                    mc_guess_stats: None,
                },
            })
        }
    }
}
