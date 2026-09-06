//! 本机玩家身份、长期统计与已结算对局记录。

use bevy::prelude::Resource;

use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerInteractionStats, PlayerReferenceChange,
    QiGui523ProfileStats, ShengjiProfileStats, TexasHoldemProfileStats, UnoProfileStats,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use super::{PlayerIdentity, config_file};

fn player_profile_path() -> Option<PathBuf> {
    config_file("profile.dat")
}

#[derive(Deserialize, Serialize)]
struct StoredPlayerProfile {
    secret_key: [u8; 32],
    games: StoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
struct StoredGameProfiles {
    qigui523: StoredRatingProfile,
    texas_holdem_stats: Option<TexasHoldemProfileStats>,
    shengji_stats: Option<ShengjiProfileStats>,
    uno_stats: Option<UnoProfileStats>,
    interaction_stats: Option<PlayerInteractionStats>,
}

#[derive(Deserialize, Serialize)]
struct StoredRatingProfile {
    reference_points: i32,
    completed_games: u32,
    applied_matches: Vec<MatchId>,
    qigui523_stats: Option<QiGui523ProfileStats>,
}

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

#[derive(Resource)]
pub struct LocalPlayerProfile {
    pub identity: PlayerIdentity,
    pub rating: PlayerRatingProfile,
    pub game_profiles: PlayerGameProfiles,
}

pub struct PlayerRatingProfile {
    pub reference_points: i32,
    pub completed_games: u32,
    pub applied_matches: HashSet<MatchId>,
    pub last_change: Option<(MatchId, i16)>,
}

impl LocalPlayerProfile {
    pub fn load_or_create() -> Result<Self, String> {
        let path = player_profile_path().ok_or_else(|| "无法确定玩家档案目录".to_owned())?;
        if path.is_file() {
            let bytes = fs::read(&path).map_err(|error| format!("无法读取玩家档案：{error}"))?;
            let stored = decode_player_profile(&bytes)?;
            return Ok(Self {
                identity: PlayerIdentity::from_secret_bytes(stored.secret_key),
                rating: PlayerRatingProfile {
                    reference_points: stored.games.qigui523.reference_points,
                    completed_games: stored.games.qigui523.completed_games,
                    applied_matches: stored.games.qigui523.applied_matches.into_iter().collect(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles {
                    qigui523: stored.games.qigui523.qigui523_stats,
                    texas_holdem: stored.games.texas_holdem_stats,
                    shengji: stored.games.shengji_stats,
                    uno: stored.games.uno_stats,
                    interactions: stored.games.interaction_stats,
                },
            });
        }

        let identity =
            PlayerIdentity::generate().map_err(|error| format!("无法生成玩家身份：{error}"))?;
        let profile = Self {
            identity,
            rating: PlayerRatingProfile {
                reference_points: 0,
                completed_games: 0,
                applied_matches: HashSet::new(),
                last_change: None,
            },
            game_profiles: PlayerGameProfiles::default(),
        };
        profile.save()?;
        Ok(profile)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = player_profile_path().ok_or_else(|| "无法确定玩家档案目录".to_owned())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建档案目录：{error}"))?;
        }
        let stored = StoredPlayerProfile {
            secret_key: self.identity.secret_bytes(),
            games: StoredGameProfiles {
                qigui523: StoredRatingProfile {
                    reference_points: self.rating.reference_points,
                    completed_games: self.rating.completed_games,
                    applied_matches: self.rating.applied_matches.iter().copied().collect(),
                    qigui523_stats: self.game_profiles.qigui523.clone(),
                },
                texas_holdem_stats: self.game_profiles.texas_holdem.clone(),
                shengji_stats: self.game_profiles.shengji.clone(),
                uno_stats: self.game_profiles.uno.clone(),
                interaction_stats: self.game_profiles.interactions.clone(),
            },
        };
        let bytes =
            postcard::to_allocvec(&stored).map_err(|error| format!("玩家档案编码失败：{error}"))?;
        fs::write(&path, bytes).map_err(|error| format!("无法保存玩家档案：{error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|error| format!("无法保护玩家档案权限：{error}"))?;
        }
        Ok(())
    }

    pub fn apply_finished_match(
        &mut self,
        match_id: MatchId,
        reference_changes: &[PlayerReferenceChange],
    ) -> bool {
        if self.rating.applied_matches.contains(&match_id) {
            return false;
        }
        let profile_id = self.identity.profile_id();
        let Some(change) = reference_changes
            .iter()
            .find(|change| change.profile_id == profile_id)
        else {
            return false;
        };
        self.rating.reference_points = self
            .rating
            .reference_points
            .saturating_add(i32::from(change.delta));
        self.rating.completed_games = self.rating.completed_games.saturating_add(1);
        self.rating.applied_matches.insert(match_id);
        self.rating.last_change = Some((match_id, change.delta));
        true
    }

    pub fn reference_points(&self) -> i32 {
        self.rating.reference_points
    }

    pub fn completed_games(&self) -> u32 {
        self.rating.completed_games
    }

    pub const fn game_profiles(&self) -> &PlayerGameProfiles {
        &self.game_profiles
    }

    pub fn sync_qigui523_profile(&mut self, stats: &QiGui523ProfileStats) -> bool {
        replace_if_changed(&mut self.game_profiles.qigui523, stats)
    }

    pub fn sync_texas_holdem_profile(&mut self, stats: &TexasHoldemProfileStats) -> bool {
        replace_if_changed(&mut self.game_profiles.texas_holdem, stats)
    }

    pub fn sync_shengji_profile(&mut self, stats: &ShengjiProfileStats) -> bool {
        replace_if_changed(&mut self.game_profiles.shengji, stats)
    }

    pub fn sync_uno_profile(&mut self, stats: &UnoProfileStats) -> bool {
        replace_if_changed(&mut self.game_profiles.uno, stats)
    }

    pub fn sync_interaction_profile(&mut self, stats: &PlayerInteractionStats) -> bool {
        replace_if_changed(&mut self.game_profiles.interactions, stats)
    }
}

fn replace_if_changed<T: Clone + PartialEq>(slot: &mut Option<T>, value: &T) -> bool {
    if slot.as_ref() == Some(value) {
        return false;
    }
    *slot = Some(value.clone());
    true
}

fn decode_player_profile(bytes: &[u8]) -> Result<StoredPlayerProfile, String> {
    match postcard::from_bytes(bytes) {
        Ok(stored) => Ok(stored),
        Err(current_error) => {
            if let Ok(previous) = postcard::from_bytes::<PreInteractionStoredPlayerProfile>(bytes) {
                return Ok(StoredPlayerProfile {
                    secret_key: previous.secret_key,
                    games: StoredGameProfiles {
                        qigui523: previous.games.qigui523,
                        texas_holdem_stats: previous.games.texas_holdem_stats,
                        shengji_stats: previous.games.shengji_stats,
                        uno_stats: previous.games.uno_stats,
                        interaction_stats: None,
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
                },
            })
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

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
}
