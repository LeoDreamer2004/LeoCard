use super::models::Node;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct WssLink {
    pub source: u64,
    pub title: String,
}

#[derive(Deserialize, Serialize)]
struct TunnelNote {
    leocard_wss: WssLink,
}

impl WssLink {
    pub(super) fn from_note(note: &str) -> Option<Self> {
        let note: TunnelNote = serde_json::from_str(note).ok()?;
        (note.leocard_wss.source != 0).then_some(note.leocard_wss)
    }

    pub(super) fn note(&self) -> Result<String, String> {
        serde_json::to_string(&TunnelNote {
            leocard_wss: self.clone(),
        })
        .map_err(|error| format!("无法编码 WSS 隧道信息：{error}"))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Tunnel {
    pub id: u64,
    pub name: String,
    pub node: u64,
    #[serde(rename = "type")]
    pub kind: String,
    pub online: bool,
    pub status: i32,
    pub status_reason: Option<String>,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub extra: String,
    pub remote: String,
    pub local_ip: String,
    pub local_port: u16,
    #[serde(skip)]
    pub(super) wss: Option<WssLink>,
    #[serde(skip)]
    pub(super) bound_domain: Option<String>,
}

impl Tunnel {
    pub fn is_wss(&self) -> bool {
        self.wss.is_some()
    }

    pub fn wss_source(&self) -> Option<u64> {
        self.wss.as_ref().map(|link| link.source)
    }

    pub fn title(&self) -> &str {
        self.wss.as_ref().map_or(&self.name, |link| &link.title)
    }

    pub fn protocol_label(&self) -> String {
        if self.is_wss() {
            "WSS".to_owned()
        } else {
            self.kind.to_ascii_uppercase()
        }
    }

    pub fn can_create_wss(&self) -> bool {
        self.kind == "tcp"
            && !self.is_wss()
            && self.extra_value("auto_https").is_none_or(str::is_empty)
    }

    pub fn address(&self, nodes: &[Node]) -> Option<String> {
        if self.is_wss() {
            let domain = self.bound_domain.as_ref()?;
            return Some(format!("wss://{domain}:{}/", self.remote));
        }
        if matches!(self.kind.as_str(), "http" | "https") {
            return Some(self.remote.clone());
        }
        let host = nodes
            .iter()
            .find(|node| node.id == self.node)?
            .host
            .as_str();
        Some(format!("{host}:{}", self.remote))
    }

    fn extra_value(&self, key: &str) -> Option<&str> {
        self.extra
            .lines()
            .filter_map(|line| line.split_once('='))
            .find_map(|(name, value)| (name.trim() == key).then(|| value.trim()))
    }

    pub(super) fn load_wss_link(&mut self) {
        self.wss = (self.kind == "tcp")
            .then(|| WssLink::from_note(&self.note))
            .flatten();
    }
}
