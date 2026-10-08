//! 本机玩家身份、长期统计与已结算对局记录。

use super::super::{PlayerIdentity, config_file};
use super::migration;
use bevy::prelude::Resource;
use leocard_protocol::{
    MahjongProfileStats, MatchId, PlayerGameProfiles, PlayerInteractionStats,
    PlayerReferenceChange, QiGui523ProfileStats, ShengjiProfileStats, TexasHoldemProfileStats,
    UnoProfileStats,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn player_profile_path() -> Option<PathBuf> {
    config_file("profile.dat")
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredPlayerProfile {
    pub(super) secret_key: [u8; 32],
    pub(super) games: StoredGameProfiles,
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredGameProfiles {
    pub(super) qigui523: StoredRatingProfile,
    pub(super) texas_holdem_stats: Option<TexasHoldemProfileStats>,
    pub(super) shengji_stats: Option<ShengjiProfileStats>,
    pub(super) uno_stats: Option<UnoProfileStats>,
    pub(super) interaction_stats: Option<PlayerInteractionStats>,
    pub(super) mahjong_stats: Option<MahjongProfileStats>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredRatingProfile {
    pub(super) reference_points: i32,
    pub(super) completed_games: u32,
    pub(super) applied_matches: Vec<MatchId>,
    pub(super) qigui523_stats: Option<QiGui523ProfileStats>,
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
            let stored = migration::decode_player_profile(&bytes)?;
            return Ok(Self {
                identity: PlayerIdentity::from_secret_bytes(stored.secret_key),
                rating: PlayerRatingProfile {
                    reference_points: stored.games.qigui523.reference_points,
                    completed_games: stored.games.qigui523.completed_games,
                    applied_matches: stored.games.qigui523.applied_matches.into_iter().collect(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles {
                    achievements: Default::default(),
                    coins: 0,
                    gender: Default::default(),
                    qigui523: stored.games.qigui523.qigui523_stats,
                    texas_holdem: stored.games.texas_holdem_stats,
                    shengji: stored.games.shengji_stats,
                    uno: stored.games.uno_stats,
                    interactions: stored.games.interaction_stats,
                    mahjong: stored.games.mahjong_stats,
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
                mahjong_stats: self.game_profiles.mahjong.clone(),
            },
        };
        let bytes =
            postcard::to_allocvec(&stored).map_err(|error| format!("玩家档案编码失败：{error}"))?;
        fs::write(&path, bytes).map_err(|error| format!("无法保存玩家档案：{error}"))?;
        #[cfg(unix)]
        {
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

    pub fn sync_game_profiles(&mut self, profiles: &PlayerGameProfiles) -> bool {
        // Achievement totals are a projection of the local achievement book.
        // Room snapshots own game statistics, never local achievement progress.
        let mut next = profiles.clone();
        next.achievements = self.game_profiles.achievements;
        next.coins = self.game_profiles.coins;
        if self.game_profiles == next {
            return false;
        }
        self.game_profiles = next;
        true
    }
}
