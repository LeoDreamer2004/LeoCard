use super::install::MinecraftLauncher;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use std::thread;
use std::time::{Duration, Instant};

const OAUTH: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0";

#[derive(Clone)]
pub struct MinecraftAccount {
    pub name: String,
    pub uuid: String,
    pub access_token: String,
    pub xuid: String,
    expires_at: Instant,
}

impl MinecraftAccount {
    pub fn expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

#[derive(Deserialize)]
struct DeviceCode {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
}

impl MinecraftLauncher {
    pub fn login_microsoft(
        &self,
        mut progress: impl FnMut(String),
    ) -> Result<MinecraftAccount, String> {
        let client_id = env::var("LEOCARD_MICROSOFT_CLIENT_ID")
            .map_err(|_| "请先设置 LEOCARD_MICROSOFT_CLIENT_ID（Azure 应用的客户端 ID，启用个人账号和公共客户端流）".to_owned())?;
        if client_id.trim().is_empty() {
            return Err("微软客户端 ID 不能为空".to_owned());
        }
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| error.to_string())?;
        let device: DeviceCode = client
            .post(format!("{OAUTH}/devicecode"))
            .form(&[
                ("client_id", client_id.as_str()),
                ("scope", "XboxLive.signin offline_access"),
            ])
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::json)
            .map_err(|error| format!("申请微软登录码失败：{error}"))?;
        progress(format!(
            "前往 {} 输入代码 {} 完成微软登录",
            device.verification_uri, device.user_code
        ));
        let deadline = Instant::now() + Duration::from_secs(device.expires_in);
        let mut interval = device.interval.unwrap_or(5).max(5);
        let microsoft_token = loop {
            if Instant::now() >= deadline {
                return Err("微软登录码已过期，请重新登录".to_owned());
            }
            thread::sleep(Duration::from_secs(interval));
            let reply: Value = client
                .post(format!("{OAUTH}/token"))
                .form(&[
                    ("client_id", client_id.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                    ("device_code", device.device_code.as_str()),
                ])
                .send()
                .and_then(reqwest::blocking::Response::json)
                .map_err(|error| format!("查询微软登录状态失败：{error}"))?;
            if let Some(token) = reply["access_token"].as_str() {
                break token.to_owned();
            }
            match reply["error"].as_str().unwrap_or_default() {
                "authorization_pending" => {}
                "slow_down" => interval += 5,
                "authorization_declined" => return Err("微软登录已被拒绝".to_owned()),
                "expired_token" => return Err("微软登录码已过期，请重新登录".to_owned()),
                error => return Err(format!("微软登录失败：{error}")),
            }
        };
        progress("验证 Xbox 和 Minecraft 账号".to_owned());
        let xbox = post_json(
            &client,
            "https://user.auth.xboxlive.com/user/authenticate",
            json!({
                "Properties": {"AuthMethod": "RPS", "SiteName": "user.auth.xboxlive.com", "RpsTicket": format!("d={microsoft_token}")},
                "RelyingParty": "http://auth.xboxlive.com", "TokenType": "JWT"
            }),
        )?;
        let xbox_token = required(&xbox, "Token")?;
        let xsts = post_json(
            &client,
            "https://xsts.auth.xboxlive.com/xsts/authorize",
            json!({
                "Properties": {"SandboxId": "RETAIL", "UserTokens": [xbox_token]},
                "RelyingParty": "rp://api.minecraftservices.com/", "TokenType": "JWT"
            }),
        )?;
        let xsts_token = required(&xsts, "Token")?;
        let user_hash = xsts["DisplayClaims"]["xui"][0]["uhs"]
            .as_str()
            .ok_or("Xbox 授权缺少用户标识")?;
        let xuid = xsts["DisplayClaims"]["xui"][0]["xid"].as_str();
        let minecraft = post_json(
            &client,
            "https://api.minecraftservices.com/authentication/login_with_xbox",
            json!({
                "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
            }),
        )?;
        let access_token = required(&minecraft, "access_token")?;
        let expires_in = minecraft["expires_in"].as_u64().unwrap_or(3600);
        let entitlements: Value = client
            .get("https://api.minecraftservices.com/entitlements/mcstore")
            .bearer_auth(&access_token)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::json)
            .map_err(|error| format!("验证 Minecraft 购买资格失败：{error}"))?;
        if entitlements["items"].as_array().is_none_or(Vec::is_empty) {
            return Err("此微软账号没有 Minecraft Java 版购买资格".to_owned());
        }
        let profile: MinecraftProfile = client
            .get("https://api.minecraftservices.com/minecraft/profile")
            .bearer_auth(&access_token)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .and_then(reqwest::blocking::Response::json)
            .map_err(|error| format!("读取 Minecraft 玩家资料失败：{error}"))?;
        Ok(MinecraftAccount {
            name: profile.name,
            uuid: profile.id,
            access_token,
            xuid: xuid
                .or_else(|| minecraft["username"].as_str())
                .unwrap_or_default()
                .to_owned(),
            expires_at: Instant::now() + Duration::from_secs(expires_in.saturating_sub(30)),
        })
    }
}

fn required(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("账号授权响应缺少 {field}"))
}

fn post_json(client: &Client, url: &str, body: Value) -> Result<Value, String> {
    let response = client
        .post(url)
        .json(&body)
        .send()
        .map_err(|error| format!("账号授权请求失败：{error}"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .map_err(|error| format!("账号授权响应无效：{error}"))?;
    if !status.is_success() {
        return Err(format!(
            "账号授权失败（{status}）：{}",
            body["XErr"]
                .as_i64()
                .map_or_else(|| body.to_string(), |code| code.to_string())
        ));
    }
    Ok(body)
}
