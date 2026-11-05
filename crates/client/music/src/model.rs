use qrcode::{Color, QrCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Deserialize, Serialize)]
pub struct Account {
    #[serde(rename = "userId")]
    pub id: u64,
    pub nickname: String,
}

// Deliberately no Debug: this contains the account's bearer credentials.
#[derive(Clone, Deserialize, Serialize)]
pub struct SavedSession {
    pub(crate) cookies: BTreeMap<String, String>,
    pub(crate) device_id: String,
    pub account: Account,
}

#[derive(Clone, Deserialize)]
pub struct Playlist {
    pub id: u64,
    pub name: String,
    #[serde(rename = "trackCount")]
    pub track_count: usize,
    pub creator: PlaylistCreator,
}

#[derive(Clone, Deserialize)]
pub struct PlaylistCreator {
    #[serde(rename = "userId")]
    pub id: u64,
}

#[derive(Clone, Deserialize)]
pub struct Song {
    pub id: u64,
    pub name: String,
    #[serde(rename = "ar")]
    pub artists: Vec<Artist>,
    #[serde(rename = "dt")]
    pub duration_ms: u64,
}

#[derive(Clone, Deserialize)]
pub struct Artist {
    pub name: Option<String>,
}

impl Song {
    pub fn artist_names(&self) -> String {
        let names = self
            .artists
            .iter()
            .filter_map(|artist| artist.name.as_deref())
            .filter(|name| !name.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" / ");
        if names.is_empty() {
            "未知歌手".to_owned()
        } else {
            names
        }
    }
}

pub struct AudioDownload {
    pub bytes: Vec<u8>,
    pub trial: bool,
}

pub enum LoginPoll {
    Waiting,
    Scanned,
    Expired,
    Authorized,
}

pub struct QrLogin {
    pub key: String,
    pub url: String,
}

impl QrLogin {
    /// RGBA pixels with a four-module quiet zone for reliable mobile scanning.
    pub fn pixels(&self) -> Result<(Vec<u8>, u32), String> {
        let qr = QrCode::new(self.url.as_bytes()).map_err(|_| "无法生成登录二维码")?;
        let side = qr.width() + 8;
        let scale = 5;
        let width = side * scale;
        let mut pixels = vec![255; width * width * 4];
        for y in 0..qr.width() {
            for x in 0..qr.width() {
                if qr[(x, y)] == Color::Dark {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let index = (((y + 4) * scale + dy) * width + (x + 4) * scale + dx) * 4;
                            pixels[index..index + 3].fill(0);
                        }
                    }
                }
            }
        }
        Ok((pixels, width as u32))
    }
}
