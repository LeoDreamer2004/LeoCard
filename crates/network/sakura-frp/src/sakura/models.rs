use super::tunnel::Tunnel;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Clone, Debug, Deserialize)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub avatar: String,
    pub group: Option<UserGroup>,
    pub traffic: Option<[u64; 2]>,
    pub tunnels: Option<u32>,
    pub ban: Option<BanInfo>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UserGroup {
    pub name: String,
    pub level: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BanInfo {
    pub title: String,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    #[serde(skip)]
    pub id: u64,
    pub name: String,
    pub host: String,
    #[serde(default)]
    pub description: String,
    pub vip: u32,
    pub flag: u64,
}

impl Node {
    pub fn can_create(&self) -> bool {
        self.flag & (1 << 2) != 0 && self.flag & (1 << 9) == 0
    }

    pub fn can_create_for(&self, user: &UserInfo) -> bool {
        self.can_create() && self.vip <= user.group.as_ref().map_or(0, |group| group.level)
    }

    pub fn supports_udp(&self) -> bool {
        self.flag & (1 << 5) != 0
    }

    pub fn allows_public_game(&self) -> bool {
        self.flag & (1 << 11) != 0 || self.name.contains("多线")
    }
}

#[derive(Clone, Debug)]
pub struct Dashboard {
    pub user: UserInfo,
    pub nodes: Vec<Node>,
    pub tunnels: Vec<Tunnel>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TunnelType {
    Tcp,
    Udp,
    Http,
    Https,
}

impl TunnelType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
            Self::Http => "HTTP",
            Self::Https => "HTTPS",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreateTunnel {
    pub name: String,
    pub kind: TunnelType,
    pub node: u64,
    pub local_ip: String,
    pub local_port: u16,
    pub remote: Option<String>,
    pub public_game: bool,
}

impl CreateTunnel {
    pub fn validate(&self, node: &Node) -> Result<(), String> {
        if self.name.trim().is_empty() || self.name.len() > 64 {
            return Err("隧道名需为 1 到 64 个字符".to_owned());
        }
        if node.id != self.node || !node.can_create() {
            return Err("所选节点不可创建隧道".to_owned());
        }
        if self.public_game && !node.allows_public_game() {
            return Err("公开游戏服务器只能使用标记支持公开游戏的节点".to_owned());
        }
        if self.kind == TunnelType::Udp && !node.supports_udp() {
            return Err("所选节点不支持 UDP".to_owned());
        }
        if self.local_ip.parse::<IpAddr>().is_err() || self.local_port == 0 {
            return Err("请输入有效的本地 IP 和 1 到 65535 的端口".to_owned());
        }
        if matches!(self.kind, TunnelType::Http | TunnelType::Https)
            && self.remote.as_deref().is_none_or(str::is_empty)
        {
            return Err("HTTP/HTTPS 隧道需要填写绑定域名".to_owned());
        }
        if matches!(self.kind, TunnelType::Tcp | TunnelType::Udp)
            && self
                .remote
                .as_deref()
                .is_some_and(|remote| remote.parse::<u16>().map_or(true, |port| port == 0))
        {
            return Err("TCP/UDP 远程端口需为 1 到 65535，或留空随机分配".to_owned());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreatedTunnel {
    pub id: u64,
    pub name: String,
    pub remote: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(flag: u64) -> Node {
        Node {
            id: 7,
            name: "测试节点".to_owned(),
            host: "example.test".to_owned(),
            description: String::new(),
            vip: 0,
            flag,
        }
    }

    fn request(kind: TunnelType, public_game: bool) -> CreateTunnel {
        CreateTunnel {
            name: "测试隧道".to_owned(),
            kind,
            node: 7,
            local_ip: "127.0.0.1".to_owned(),
            local_port: 25565,
            remote: Some("25565".to_owned()),
            public_game,
        }
    }

    #[test]
    fn public_game_requires_the_public_game_node_flag() {
        let node = make_node((1 << 2) | (1 << 5));
        assert!(request(TunnelType::Tcp, false).validate(&node).is_ok());
        assert!(request(TunnelType::Tcp, true).validate(&node).is_err());
        assert!(
            request(TunnelType::Tcp, true)
                .validate(&make_node((1 << 2) | (1 << 5) | (1 << 11)))
                .is_ok()
        );
    }

    #[test]
    fn udp_requires_node_support_and_a_valid_remote_port() {
        let node = make_node(1 << 2);
        assert!(request(TunnelType::Udp, false).validate(&node).is_err());
        let node = make_node((1 << 2) | (1 << 5));
        assert!(request(TunnelType::Udp, false).validate(&node).is_ok());
        let mut request = request(TunnelType::Udp, false);
        request.remote = Some("70000".to_owned());
        assert!(request.validate(&node).is_err());
    }
}
