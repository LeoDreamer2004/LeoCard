use crate::app::runtime::valid_normalized_avatar;
use crate::network::normalize_server_address;
use crate::player::config_file;
use bevy::log::warn;
use leocard_platform::storage as fs;
use leocard_protocol::{AvatarId, GameKind, RoomInfo};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct CachedHost {
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub image: Option<String>,
    pub game: GameKind,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct SavedRoom {
    pub name: String,
    pub address: String,
    pub host: Option<CachedHost>,
}

pub(crate) struct SavedRoomStore {
    pub entries: Vec<SavedRoom>,
    path: Option<PathBuf>,
}

impl Default for SavedRoomStore {
    fn default() -> Self {
        Self::load(config_file("rooms.json"))
    }
}

impl SavedRoomStore {
    pub fn load(path: Option<PathBuf>) -> Self {
        let entries = path
            .as_ref()
            .and_then(|path| match fs::read(path) {
                Ok(bytes) => match serde_json::from_slice::<Vec<SavedRoom>>(&bytes) {
                    Ok(entries) => Some(entries),
                    Err(error) => {
                        warn!("无法读取已保存房间：{error}");
                        None
                    }
                },
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(error) => {
                    warn!("无法读取已保存房间：{error}");
                    None
                }
            })
            .unwrap_or_default();
        Self { entries, path }
    }

    pub fn save(
        &mut self,
        original: Option<&str>,
        name: &str,
        address: &str,
    ) -> Result<String, String> {
        let address = normalize_server_address(address).map_err(|error| error.to_string())?;
        let name = name.trim();
        if name.chars().count() > 40 {
            return Err("房间名称不能超过 40 个字".to_owned());
        }
        if self
            .entries
            .iter()
            .any(|room| room.address == address && Some(room.address.as_str()) != original)
        {
            return Err("这个房间地址已保存".to_owned());
        }
        let mut entries = self.entries.clone();
        let mut room = SavedRoom {
            name: if name.is_empty() {
                address.clone()
            } else {
                name.to_owned()
            },
            address: address.clone(),
            host: None,
        };
        if let Some(index) =
            original.and_then(|old| entries.iter().position(|room| room.address == old))
        {
            if entries[index].address == address {
                room.host.clone_from(&entries[index].host);
            }
            entries[index] = room;
        } else {
            entries.push(room);
        }
        self.commit(entries)?;
        Ok(address)
    }

    pub fn remove(&mut self, address: &str) -> Result<(), String> {
        let mut entries = self.entries.clone();
        entries.retain(|room| room.address != address);
        self.commit(entries)
    }

    pub fn remember(&mut self, address: &str, info: &RoomInfo) -> Result<bool, String> {
        let address = normalize_server_address(address).map_err(|error| error.to_string())?;
        let png = info
            .avatar_png
            .as_deref()
            .filter(|png| valid_normalized_avatar(png));
        let image = png
            .map(|png| {
                Sha256::digest(png)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            })
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|room| room.address == address)?
                    .host
                    .as_ref()
                    .filter(|host| {
                        host.avatar == info.host_avatar
                            && host.name == info.host_name
                            && info.host_avatar.is_some()
                    })?
                    .image
                    .clone()
            });
        let host = CachedHost {
            name: info.host_name.clone(),
            avatar: info.host_avatar,
            image: image.clone(),
            game: info.game,
        };
        if let Some(image) = image
            && let Some(png) = png
            && let Some(path) = self.avatar_path(&image)
            && !fs::is_file(&path)
        {
            fs::create_dir_all(path.parent().expect("avatar directory"))
                .map_err(|error| error.to_string())?;
            fs::write(path, png).map_err(|error| error.to_string())?;
        }
        if self
            .entries
            .iter()
            .any(|room| room.address == address && room.host.as_ref() == Some(&host))
        {
            return Ok(false);
        }
        let mut entries = self.entries.clone();
        if let Some(room) = entries.iter_mut().find(|room| room.address == address) {
            room.host = Some(host);
        } else {
            entries.push(SavedRoom {
                name: format!("{}的房间", info.host_name),
                address,
                host: Some(host),
            });
        }
        self.commit(entries)?;
        Ok(true)
    }

    pub fn avatar(&self, key: &str) -> Option<Vec<u8>> {
        let png = fs::read(self.avatar_path(key)?).ok()?;
        valid_normalized_avatar(&png).then_some(png)
    }

    fn avatar_path(&self, key: &str) -> Option<PathBuf> {
        if key.len() != 64 || !key.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        Some(
            self.path
                .as_ref()?
                .parent()?
                .join("room-avatars")
                .join(format!("{key}.png")),
        )
    }

    fn commit(&mut self, entries: Vec<SavedRoom>) -> Result<(), String> {
        let path = self.path.as_ref().ok_or("无法确定房间列表存储位置")?;
        fs::create_dir_all(path.parent().ok_or("房间列表路径无效")?)
            .map_err(|error| error.to_string())?;
        let bytes = serde_json::to_vec_pretty(&entries).map_err(|error| error.to_string())?;
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
        fs::rename(temporary, path).map_err(|error| error.to_string())?;
        self.entries = entries;
        Ok(())
    }
}
