use super::crypto::encrypt_request;
use reqwest::blocking::{Client, Response};
use reqwest::header::{COOKIE, SET_COOKIE};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(super) struct Transport {
    pub http: Client,
    pub cookies: Arc<Mutex<BTreeMap<String, String>>>,
    pub device_id: String,
}

impl Transport {
    pub fn new(cookies: BTreeMap<String, String>, device_id: String) -> Result<Self, String> {
        let http = Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(25))
            .user_agent("NeteaseMusic/3.0.18.203152")
            .build()
            .map_err(|_| "无法初始化音乐网络连接")?;
        Ok(Self {
            http,
            cookies: Arc::new(Mutex::new(cookies)),
            device_id,
        })
    }

    pub fn request(&self, path: &str, mut data: Value) -> Result<Value, String> {
        let cookies = self
            .cookies
            .lock()
            .map_err(|_| "音乐登录状态不可用")?
            .clone();
        let mut header = json!({
            "os": "pc", "appver": "3.0.18", "deviceId": self.device_id,
            "requestId": format!("{}_{:04}", now_ms(), fastrand::u16(0..10000)),
            "__csrf": cookies.get("__csrf").cloned().unwrap_or_default(),
        });
        for name in ["MUSIC_U", "MUSIC_A", "NMTID"] {
            if let Some(value) = cookies.get(name) {
                header[name] = json!(value);
            }
        }
        data["header"] = header;
        data["e_r"] = json!(false);
        let cookie_header = cookies
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("; ");
        let params = encrypt_request(path, &data.to_string());
        let response = self
            .http
            .post(format!(
                "https://interface.music.163.com/eapi/{}",
                &path[5..]
            ))
            .header("Referer", "https://music.163.com/")
            .header(COOKIE, cookie_header)
            .form(&[("params", params)])
            .send()
            .and_then(Response::error_for_status)
            .map_err(|_| "网易云请求失败，请检查网络后重试")?;
        {
            let mut cookies = self.cookies.lock().map_err(|_| "音乐登录状态不可用")?;
            for cookie in response.headers().get_all(SET_COOKIE) {
                if let Ok(cookie) = cookie.to_str()
                    && let Some((name, value)) = cookie
                        .split(';')
                        .next()
                        .and_then(|pair| pair.split_once('='))
                {
                    cookies.insert(name.trim().to_owned(), value.to_owned());
                }
            }
        }
        let bytes = bounded_bytes(response, 8 * 1024 * 1024)?;
        let value: Value =
            serde_json::from_slice(&bytes).map_err(|_| "网易云返回了无法识别的数据")?;
        match value["code"].as_i64() {
            Some(200 | 800..=803) => Ok(value),
            Some(301 | 302) => Err("网易云登录已失效，请解除关联后重新扫码".to_owned()),
            Some(code) => Err(format!("网易云暂时无法完成请求（{code}），请稍后重试")),
            None => Err("网易云响应缺少状态码".to_owned()),
        }
    }
}

pub(super) fn bounded_bytes(response: Response, maximum: u64) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum)
    {
        return Err("音乐数据超过大小限制".to_owned());
    }
    let mut bytes = Vec::new();
    response
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "音乐数据接收中断，请重试")?;
    if bytes.len() as u64 > maximum {
        return Err("音乐数据超过大小限制".to_owned());
    }
    Ok(bytes)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
