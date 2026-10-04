//! Versioned achievement archives. Game releases do not change the format version.

use crate::player::config_file;
use leocard_achievements::AchievementBook;
use leocard_protocol::ProfileId;
use serde::{Deserialize, Serialize};
use std::{fs, io::ErrorKind, path::PathBuf};

const FILE_MAGIC: &[u8] = b"LEOCARD-ACHIEVEMENTS\n";
// Increase only when the archive or AchievementBook layout changes, and add a migration.
const FORMAT_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
pub(super) struct AchievementArchive {
    profile_id: ProfileId,
    pub(super) book: AchievementBook,
}

impl AchievementArchive {
    pub(super) fn new(profile_id: ProfileId) -> Self {
        Self {
            profile_id,
            book: AchievementBook::default(),
        }
    }

    pub(super) fn load(profile_id: ProfileId) -> Result<Self, String> {
        let bytes = match fs::read(Self::path()?) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::new(profile_id)),
            Err(error) => return Err(format!("无法读取成就档案：{error}")),
        };
        let stored = Self::decode(&bytes)?;
        if stored.profile_id != profile_id {
            return Ok(Self::new(profile_id));
        }
        Ok(stored)
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        let body = bytes
            .strip_prefix(FILE_MAGIC)
            .ok_or_else(|| "无法识别成就档案格式，原存档已保留".to_owned())?;
        let Some((version, payload)) = body.split_first_chunk::<2>() else {
            return Err("成就档案格式版本不完整，原存档已保留".to_owned());
        };
        let version = u16::from_be_bytes(*version);
        match version {
            FORMAT_VERSION => Self::decode_payload(payload),
            _ => Err(format!("不支持成就档案格式版本 {version}，原存档已保留")),
        }
    }

    fn decode_payload(payload: &[u8]) -> Result<Self, String> {
        postcard::from_bytes(payload).map_err(|error| format!("无法解析成就档案：{error}"))
    }

    pub(super) fn encode(&self) -> Result<Vec<u8>, String> {
        let mut bytes = FILE_MAGIC.to_vec();
        bytes.extend(FORMAT_VERSION.to_be_bytes());
        bytes.extend(
            postcard::to_allocvec(self).map_err(|error| format!("成就档案编码失败：{error}"))?,
        );
        Ok(bytes)
    }

    pub(super) fn save(&self) -> Result<(), String> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("无法创建成就档案目录：{error}"))?;
        }
        let temporary = path.with_extension("dat.tmp");
        fs::write(&temporary, self.encode()?)
            .map_err(|error| format!("无法保存成就档案：{error}"))?;
        fs::rename(temporary, path).map_err(|error| format!("无法更新成就档案：{error}"))
    }

    fn path() -> Result<PathBuf, String> {
        config_file("achievements.dat").ok_or_else(|| "无法确定成就档案目录".to_owned())
    }
}
