use super::models::{CreateTunnel, CreatedTunnel, Dashboard, Node, UserInfo};
use super::tunnel::Tunnel;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

const API: &str = "https://api.natfrp.com/v4";

#[derive(Clone)]
pub struct SakuraClient {
    pub(super) http: Client,
    token: String,
    endpoint: String,
}

#[derive(Deserialize)]
struct ApiError {
    msg: Option<String>,
}

#[derive(Deserialize)]
struct DeleteResponse {
    deleted: Vec<u64>,
    failed: Vec<u64>,
}

impl SakuraClient {
    pub fn new(token: String) -> Result<Self, String> {
        let token = token.trim().to_owned();
        if token.is_empty() || token.len() > 256 || token.chars().any(char::is_whitespace) {
            return Err("访问密钥格式无效".to_owned());
        }
        let http = Client::builder()
            .user_agent(format!("LeoCard/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            http,
            token,
            endpoint: API.to_owned(),
        })
    }

    pub fn dashboard(&self) -> Result<Dashboard, String> {
        let user = self.user_info()?;
        let nodes: HashMap<String, Node> = self.get("/nodes")?;
        let mut nodes = nodes
            .into_iter()
            .filter_map(|(id, mut node)| {
                node.id = id.parse().ok()?;
                Some(node)
            })
            .collect::<Vec<_>>();
        nodes.sort_by_key(|node| node.id);
        let mut tunnels: Vec<Tunnel> = self.get("/tunnels")?;
        tunnels.sort_by_key(|tunnel| tunnel.id);
        self.load_wss_addresses(&mut tunnels)?;
        Ok(Dashboard {
            user,
            nodes,
            tunnels,
        })
    }

    pub fn user_info(&self) -> Result<UserInfo, String> {
        let user: UserInfo = self.get("/user/info")?;
        if let Some(ban) = &user.ban {
            return Err(format!(
                "SakuraFrp 账号已冻结：{} {}",
                ban.title, ban.reason
            ));
        }
        Ok(user)
    }

    pub fn avatar(&self, user: &UserInfo) -> Result<Vec<u8>, String> {
        let url = reqwest::Url::parse(&user.avatar).map_err(|_| "头像地址无效")?;
        if url.scheme() != "https" {
            return Err("头像地址必须使用 HTTPS".to_owned());
        }
        // Public image requests never carry the account's bearer credential.
        let response = self
            .http
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(|_| "下载 SakuraFrp 头像失败")?;
        let mut bytes = Vec::new();
        response
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| "读取 SakuraFrp 头像失败")?;
        if bytes.len() > 1_048_576 {
            return Err("SakuraFrp 头像过大".to_owned());
        }
        Ok(bytes)
    }

    pub fn create_tunnel(
        &self,
        request: &CreateTunnel,
        node: &Node,
    ) -> Result<CreatedTunnel, String> {
        self.create_tunnel_with_extra(request, node, "", "")
    }

    pub(super) fn create_tunnel_with_extra(
        &self,
        request: &CreateTunnel,
        node: &Node,
        note: &str,
        extra: &str,
    ) -> Result<CreatedTunnel, String> {
        request.validate(node)?;
        let mut body = json!({
            "name": request.name.trim(),
            "type": request.kind,
            "node": request.node,
            "local_ip": request.local_ip,
            "local_port": request.local_port,
            "note": note,
            "extra": extra,
        });
        if let Some(remote) = &request.remote {
            body["remote"] = json!(remote);
        }
        self.json(
            self.authorized(self.http.post(self.url("/tunnels")))
                .json(&body),
        )
    }

    pub fn delete_tunnel(&self, id: u64) -> Result<bool, String> {
        let reply: DeleteResponse = self.json(
            self.authorized(self.http.post(self.url("/tunnel/delete")))
                .json(&json!({ "ids": id.to_string() })),
        )?;
        if !reply.deleted.contains(&id) && !reply.failed.contains(&id) {
            return Err(format!("隧道 {id} 未被删除"));
        }
        Ok(reply.failed.contains(&id))
    }

    pub(super) fn config(&self, id: u64, frpc_version: &str) -> Result<String, String> {
        let request = self
            .authorized(self.http.post(self.url("/tunnel/config")))
            .form(&[("query", id.to_string()), ("frpc", frpc_version.to_owned())]);
        let response = request
            .send()
            .map_err(|error| format!("获取隧道配置失败：{error}"))?;
        let status = response.status();
        let body = response.text().map_err(|error| error.to_string())?;
        if status.is_success() {
            Ok(body)
        } else {
            let message = serde_json::from_str::<ApiError>(&body)
                .ok()
                .and_then(|error| error.msg)
                .unwrap_or_else(|| status.to_string());
            Err(format!("获取隧道配置失败：{message}"))
        }
    }

    pub(super) fn public_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.json(self.http.get(self.url(path)))
    }

    pub(super) fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, String> {
        self.json(self.authorized(self.http.get(self.url(path))))
    }

    pub(super) fn authorized(&self, request: RequestBuilder) -> RequestBuilder {
        request.bearer_auth(&self.token)
    }

    pub(super) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.endpoint)
    }

    #[cfg(test)]
    pub(super) fn with_test_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = endpoint;
        self
    }

    pub(super) fn json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, String> {
        self.response(request)?
            .json()
            .map_err(|error| format!("SakuraFrp 响应无效：{error}"))
    }

    pub(super) fn empty(&self, request: RequestBuilder) -> Result<(), String> {
        self.response(request).map(|_| ())
    }

    fn response(&self, request: RequestBuilder) -> Result<Response, String> {
        let response = request
            .send()
            .map_err(|error| format!("SakuraFrp 请求失败：{error}"))?;
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            let message = response
                .json::<ApiError>()
                .ok()
                .and_then(|error| error.msg)
                .unwrap_or_else(|| status.to_string());
            Err(format!("SakuraFrp 请求失败：{message}"))
        }
    }
}
