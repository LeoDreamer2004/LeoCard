use super::{PropId, migration};
use crate::player::config_file;
use leocard_protocol::{AvatarFrameId, MatchId, ProfileId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::io::ErrorKind;

const FILE_MAGIC: &[u8] = b"LEOCARD-ECONOMY\n";
pub(super) const FORMAT_VERSION: u16 = 5;

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct EconomyArchive {
    pub ledger: EconomyLedger,
    pub owned_frames: BTreeSet<AvatarFrameId>,
    pub equipped_frame: AvatarFrameId,
    pub exclusive_frame: Option<AvatarFrameId>,
}

/// The original v3 wallet payload, reused by later formats.
#[derive(Clone, Deserialize, Serialize)]
pub(super) struct EconomyLedger {
    pub profile_id: ProfileId,
    pub coins: u32,
    pub active_props: BTreeMap<PropId, u64>,
    pub last_login_day: Option<u64>,
    pub rewarded_achievements: BTreeSet<String>,
    pub settled_matches: HashSet<MatchId>,
    pub rewards_initialized: bool,
}

impl EconomyArchive {
    pub fn new(profile_id: ProfileId) -> Self {
        Self::with_ledger(EconomyLedger {
            profile_id,
            coins: 0,
            active_props: BTreeMap::new(),
            last_login_day: None,
            rewarded_achievements: BTreeSet::new(),
            settled_matches: HashSet::new(),
            rewards_initialized: false,
        })
    }

    pub(super) fn with_ledger(ledger: EconomyLedger) -> Self {
        Self {
            ledger,
            owned_frames: BTreeSet::new(),
            equipped_frame: AvatarFrameId::Default,
            exclusive_frame: None,
        }
    }

    pub(super) fn owns_frame(&self, frame: AvatarFrameId) -> bool {
        frame == AvatarFrameId::Default || self.owned_frames.contains(&frame)
    }

    pub(super) fn can_use_frame(&self, frame: AvatarFrameId) -> bool {
        self.owns_frame(frame) || self.exclusive_frame == Some(frame)
    }

    pub(super) fn frame_for_exclusive(&self, exclusive: Option<AvatarFrameId>) -> AvatarFrameId {
        match (self.exclusive_frame, exclusive) {
            (previous, Some(frame)) if previous != exclusive => frame,
            (Some(previous), None) if !self.owns_frame(previous) => AvatarFrameId::Default,
            _ if self.owns_frame(self.equipped_frame) || exclusive == Some(self.equipped_frame) => {
                self.equipped_frame
            }
            _ => AvatarFrameId::Default,
        }
    }

    pub(super) fn update_exclusive_frame(&mut self, exclusive: Option<AvatarFrameId>) -> bool {
        if self.exclusive_frame == exclusive {
            return false;
        }
        self.equipped_frame = self.frame_for_exclusive(exclusive);
        self.exclusive_frame = exclusive;
        true
    }

    pub fn load(profile_id: ProfileId, exclusive: Option<AvatarFrameId>) -> Result<Self, String> {
        let path = config_file("economy.dat").ok_or("无法确定钱包目录")?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let mut archive = Self::new(profile_id);
                archive.update_exclusive_frame(exclusive);
                archive.save()?;
                return Ok(archive);
            }
            Err(error) => return Err(format!("无法读取钱包：{error}")),
        };
        let body = bytes
            .strip_prefix(FILE_MAGIC)
            .ok_or("无法识别钱包格式，原存档已保留")?;
        let (version, payload) = body.split_first_chunk::<2>().ok_or("钱包格式版本不完整")?;
        let version = u16::from_be_bytes(*version);
        let mut archive = migration::decode(version, payload, profile_id, exclusive)?;
        let changed = archive.update_exclusive_frame(exclusive);
        if version != FORMAT_VERSION || changed {
            archive.save()?;
        }
        Ok(archive)
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_file("economy.dat").ok_or("无法确定钱包目录")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建钱包目录：{error}"))?;
        }
        let mut bytes = FILE_MAGIC.to_vec();
        bytes.extend(FORMAT_VERSION.to_be_bytes());
        bytes
            .extend(postcard::to_allocvec(self).map_err(|error| format!("钱包编码失败：{error}"))?);
        let temporary = path.with_extension("dat.tmp");
        fs::write(&temporary, bytes).map_err(|error| format!("无法保存钱包：{error}"))?;
        fs::rename(temporary, path).map_err(|error| format!("无法更新钱包：{error}"))
    }
}
