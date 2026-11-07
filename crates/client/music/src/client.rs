use super::transport::{Transport, bounded_bytes};
use super::{Account, AudioDownload, LoginPoll, Playlist, QrLogin, SavedSession, Song};
use reqwest::Url;
use reqwest::blocking::Response;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone)]
pub struct MusicClient {
    transport: Transport,
}

impl MusicClient {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            transport: Transport::new(BTreeMap::new(), format!("{:032x}", fastrand::u128(..)))?,
        })
    }

    pub fn restore(saved: &SavedSession) -> Result<Self, String> {
        Ok(Self {
            transport: Transport::new(saved.cookies.clone(), saved.device_id.clone())?,
        })
    }

    pub fn session(&self, account: Account) -> Result<SavedSession, String> {
        Ok(SavedSession {
            cookies: self
                .transport
                .cookies
                .lock()
                .map_err(|_| "音乐登录状态不可用")?
                .clone(),
            device_id: self.transport.device_id.clone(),
            account,
        })
    }

    pub fn qr_login(&self) -> Result<QrLogin, String> {
        let response = self
            .transport
            .request("/api/login/qrcode/unikey", json!({"type": 3}))?;
        let key = response["unikey"]
            .as_str()
            .filter(|key| !key.is_empty())
            .ok_or("未收到登录二维码")?
            .to_owned();
        let mut url = Url::parse("https://music.163.com/login").map_err(|_| "登录地址无效")?;
        url.query_pairs_mut().append_pair("codekey", &key);
        Ok(QrLogin {
            key,
            url: url.to_string(),
        })
    }

    pub fn poll_login(&self, key: &str) -> Result<LoginPoll, String> {
        let value = self.transport.request(
            "/api/login/qrcode/client/login",
            json!({"key": key, "type": 3}),
        )?;
        match value["code"].as_u64() {
            Some(800) => Ok(LoginPoll::Expired),
            Some(801) => Ok(LoginPoll::Waiting),
            Some(802) => Ok(LoginPoll::Scanned),
            Some(803) => Ok(LoginPoll::Authorized),
            _ => Err("二维码登录状态异常，请刷新二维码".to_owned()),
        }
    }

    pub fn account(&self) -> Result<Account, String> {
        let value = self
            .transport
            .request("/api/w/nuser/account/get", json!({}))?;
        decode(&value["profile"]).map_err(|_| "未读取到网易云账号，请重新扫码".to_owned())
    }

    pub fn playlists(&self, user_id: u64) -> Result<Vec<Playlist>, String> {
        let mut playlists = Vec::new();
        loop {
            let value = self.transport.request("/api/user/playlist", json!({"uid": user_id, "limit": 1000, "offset": playlists.len(), "includeVideo": false}))?;
            let page: Vec<Playlist> = decode(&value["playlist"])?;
            let more = value["more"].as_bool().unwrap_or(false);
            if page.is_empty() {
                if more {
                    return Err("网易云未返回下一页歌单，请重试".to_owned());
                }
                break;
            }
            playlists.extend(page);
            if !more {
                break;
            }
        }
        Ok(playlists)
    }

    pub fn songs(&self, playlist_id: u64) -> Result<Vec<Song>, String> {
        let value = self.transport.request(
            "/api/v6/playlist/detail",
            json!({"id": playlist_id, "n": 100000, "s": 0}),
        )?;
        let ids: Vec<u64> = value["playlist"]["trackIds"]
            .as_array()
            .ok_or("未收到歌单曲目")?
            .iter()
            .map(|track| {
                track["id"]
                    .as_u64()
                    .ok_or_else(|| "歌单曲目编号无效".to_owned())
            })
            .collect::<Result<_, _>>()?;
        let mut songs = BTreeMap::new();
        for chunk in ids.chunks(500) {
            let c = chunk.iter().map(|id| json!({"id": id})).collect::<Vec<_>>();
            let value = self.transport.request(
                "/api/v3/song/detail",
                json!({"c": serde_json::to_string(&c).map_err(|_| "无法读取曲目")?}),
            )?;
            for song in decode::<Vec<Song>>(&value["songs"])? {
                songs.insert(song.id, song);
            }
        }
        // Preserve the user's playlist order, even if song/detail reorders its response.
        Ok(ids.into_iter().filter_map(|id| songs.remove(&id)).collect())
    }

    pub fn audio(&self, song_id: u64) -> Result<AudioDownload, String> {
        let value = self.transport.request(
            "/api/song/enhance/player/url/v1",
            json!({"ids": format!("[{song_id}]"), "level": "standard", "encodeType": "mp3"}),
        )?;
        let song = value["data"]
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item["id"].as_u64() == Some(song_id))
            })
            .ok_or("未收到歌曲播放信息")?;
        let raw_url = song["url"]
            .as_str()
            .filter(|url| !url.is_empty())
            .ok_or("该歌曲暂无播放权限或已下架，请选择其他歌曲")?;
        let mut url = Url::parse(raw_url).map_err(|_| "歌曲播放地址无效")?;
        if url.scheme() == "http" {
            url.set_scheme("https").map_err(|_| "歌曲播放地址无效")?;
        }
        // The download request never carries account cookies; only NetEase API requests do.
        let response = self
            .transport
            .http
            .get(url)
            .timeout(Duration::from_secs(90))
            .send()
            .and_then(Response::error_for_status)
            .map_err(|_| "歌曲加载失败，请重试")?;
        let bytes = bounded_bytes(response, 32 * 1024 * 1024)?;
        if bytes.is_empty() {
            return Err("网易云返回了空的歌曲数据".to_owned());
        }
        Ok(AudioDownload {
            bytes,
            trial: !song["freeTrialInfo"].is_null(),
        })
    }
}

fn decode<T: DeserializeOwned>(value: &Value) -> Result<T, String> {
    serde_json::from_value(value.clone())
        .map_err(|_| "网易云返回的数据格式已变化，请稍后重试".to_owned())
}
