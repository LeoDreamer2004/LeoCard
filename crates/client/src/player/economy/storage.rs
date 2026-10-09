use super::{ItemId, migration};
use crate::player::config_file;
use leocard_protocol::{MatchId, ProfileId};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fs,
    io::ErrorKind,
};

const FILE_MAGIC: &[u8] = b"LEOCARD-ECONOMY\n";
pub(super) const FORMAT_VERSION: u16 = 3;

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct EconomyArchive {
    pub profile_id: ProfileId,
    pub coins: u32,
    pub active_items: BTreeMap<ItemId, u64>,
    pub last_login_day: Option<u64>,
    pub rewarded_achievements: BTreeSet<String>,
    pub settled_matches: HashSet<MatchId>,
    pub rewards_initialized: bool,
}

impl EconomyArchive {
    pub fn new(profile_id: ProfileId) -> Self {
        Self {
            profile_id,
            coins: 0,
            active_items: BTreeMap::new(),
            last_login_day: None,
            rewarded_achievements: BTreeSet::new(),
            settled_matches: HashSet::new(),
            rewards_initialized: false,
        }
    }

    pub fn load(profile_id: ProfileId) -> Result<Self, String> {
        let path = config_file("economy.dat").ok_or("无法确定钱包目录")?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let archive = Self::new(profile_id);
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
        let archive = migration::decode(version, payload, profile_id)?;
        if version != FORMAT_VERSION {
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
