//! 本机玩家身份、长期统计与已结算对局记录。

use super::super::{PlayerIdentity, config_file};
use super::migration;
use bevy::prelude::Resource;
use leocard_blackjack::BlackjackProfileStats;
use leocard_doudizhu::DoudizhuProfileStats;
use leocard_guandan::GuandanProfileStats;
use leocard_mahjong::MahjongProfileStats;
use leocard_mc_guess::McGuessProfileStats;
use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerInteractionStats, PlayerReferenceChange,
};
use leocard_qigui523::QiGui523ProfileStats;
use leocard_shengji::ShengjiProfileStats;
use leocard_texas_holdem::TexasProfileStats;
use leocard_uno::UnoProfileStats;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

pub(super) const PROFILE_HEADER: &[u8] = b"LeoCard/profile/v8\0";

fn player_profile_path() -> Option<PathBuf> {
    config_file("profile.dat")
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredPlayerProfile {
    pub(super) secret_key: [u8; 32],
    pub(super) rating: StoredRatingProfile,
    pub(super) interaction_stats: Option<PlayerInteractionStats>,
    pub(super) games: StoredGameProfiles,
}

impl StoredPlayerProfile {
    pub(super) fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        let mut bytes = PROFILE_HEADER.to_vec();
        bytes.extend_from_slice(&postcard::to_allocvec(self)?);
        Ok(bytes)
    }

    fn write(&self, path: &Path) -> Result<(), String> {
        let bytes = self
            .encode()
            .map_err(|error| format!("玩家档案编码失败：{error}"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建档案目录：{error}"))?;
        }
        let temporary = path.with_extension(format!("dat.{:016x}.tmp", fastrand::u64(..)));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(&temporary)
            .map_err(|error| format!("无法创建临时玩家档案：{error}"))?;
        let result = file.write_all(&bytes).and_then(|()| file.sync_all());
        drop(file);
        let result = result.and_then(|()| fs::rename(&temporary, path));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(|error| format!("无法保存玩家档案：{error}"))
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredGameProfiles {
    pub(super) qigui523_stats: Option<QiGui523ProfileStats>,
    pub(super) texas_stats: Option<TexasProfileStats>,
    pub(super) shengji_stats: Option<ShengjiProfileStats>,
    pub(super) uno_stats: Option<UnoProfileStats>,
    pub(super) mahjong_stats: Option<MahjongProfileStats>,
    pub(super) mc_guess_stats: Option<McGuessProfileStats>,
    pub(super) blackjack_stats: Option<BlackjackProfileStats>,
    pub(super) doudizhu_stats: Option<DoudizhuProfileStats>,
    pub(super) guandan_stats: Option<GuandanProfileStats>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct StoredRatingProfile {
    pub(super) reference_points: i32,
    pub(super) completed_games: u32,
    pub(super) applied_matches: Vec<MatchId>,
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
        Self::load_or_create_at(&path)
    }

    pub(super) fn load_or_create_at(path: &Path) -> Result<Self, String> {
        if path.is_file() {
            let bytes = fs::read(path).map_err(|error| format!("无法读取玩家档案：{error}"))?;
            let stored = migration::decode_player_profile(&bytes)?;
            if !bytes.starts_with(PROFILE_HEADER) {
                stored.write(path)?;
            }
            return Ok(Self {
                identity: PlayerIdentity::from_secret_bytes(stored.secret_key),
                rating: PlayerRatingProfile {
                    reference_points: stored.rating.reference_points,
                    completed_games: stored.rating.completed_games,
                    applied_matches: stored.rating.applied_matches.into_iter().collect(),
                    last_change: None,
                },
                game_profiles: PlayerGameProfiles {
                    achievements: Default::default(),
                    coins: 0,
                    gender: Default::default(),
                    avatar_frame: Default::default(),
                    qigui523: stored.games.qigui523_stats,
                    texas: stored.games.texas_stats,
                    shengji: stored.games.shengji_stats,
                    uno: stored.games.uno_stats,
                    interactions: stored.interaction_stats,
                    mahjong: stored.games.mahjong_stats,
                    mc_guess: stored.games.mc_guess_stats,
                    blackjack: stored.games.blackjack_stats,
                    doudizhu: stored.games.doudizhu_stats,
                    guandan: stored.games.guandan_stats,
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
        profile.store().write(path)?;
        Ok(profile)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = player_profile_path().ok_or_else(|| "无法确定玩家档案目录".to_owned())?;
        self.store().write(&path)
    }

    fn store(&self) -> StoredPlayerProfile {
        StoredPlayerProfile {
            secret_key: self.identity.secret_bytes(),
            rating: StoredRatingProfile {
                reference_points: self.rating.reference_points,
                completed_games: self.rating.completed_games,
                applied_matches: self.rating.applied_matches.iter().copied().collect(),
            },
            interaction_stats: self.game_profiles.interactions.clone(),
            games: StoredGameProfiles {
                qigui523_stats: self.game_profiles.qigui523.clone(),
                texas_stats: self.game_profiles.texas.clone(),
                shengji_stats: self.game_profiles.shengji.clone(),
                uno_stats: self.game_profiles.uno.clone(),
                mahjong_stats: self.game_profiles.mahjong.clone(),
                mc_guess_stats: self.game_profiles.mc_guess.clone(),
                blackjack_stats: self.game_profiles.blackjack.clone(),
                doudizhu_stats: self.game_profiles.doudizhu.clone(),
                guandan_stats: self.game_profiles.guandan.clone(),
            },
        }
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
        next.avatar_frame = self.game_profiles.avatar_frame;
        if self.game_profiles == next {
            return false;
        }
        self.game_profiles = next;
        true
    }
}
