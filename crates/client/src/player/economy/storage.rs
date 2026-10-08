use super::ItemId;
use crate::player::config_file;
use leocard_protocol::ProfileId;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::ErrorKind};

const FILE_MAGIC: &[u8] = b"LEOCARD-ECONOMY\n";
const FORMAT_VERSION: u16 = 2;

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct EconomyArchive {
    pub profile_id: ProfileId,
    pub coins: u32,
    pub active_items: BTreeMap<ItemId, u64>,
}

impl EconomyArchive {
    pub fn new(profile_id: ProfileId) -> Self {
        Self {
            profile_id,
            coins: 300,
            active_items: BTreeMap::new(),
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
        if u16::from_be_bytes(*version) == 1 {
            let archive = Self::new(profile_id);
            archive.save()?;
            return Ok(archive);
        }
        if u16::from_be_bytes(*version) != FORMAT_VERSION {
            return Err("不支持此钱包版本，原存档已保留".to_owned());
        }
        let archive: Self =
            postcard::from_bytes(payload).map_err(|error| format!("无法解析钱包：{error}"))?;
        if archive.profile_id != profile_id {
            return Err("钱包与当前玩家身份不匹配，原存档已保留".to_owned());
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
