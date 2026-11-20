use super::api::SakuraClient;
use super::models::{CreateTunnel, CreatedTunnel, Node, TunnelType};
use super::tunnel::{Tunnel, WssLink};
use std::collections::HashMap;

const WSS_EXTRA: &str = "auto_https = auto\nauto_https_mode = passthrough";

impl SakuraClient {
    pub(super) fn load_wss_addresses(&self, tunnels: &mut [Tunnel]) -> Result<(), String> {
        for tunnel in tunnels.iter_mut() {
            tunnel.load_wss_link();
        }
        if !tunnels.iter().any(Tunnel::is_wss) {
            return Ok(());
        }
        self.domain_settings()?.assign(tunnels);
        let titles = tunnels
            .iter()
            .map(|tunnel| (tunnel.id, tunnel.name.clone()))
            .collect::<HashMap<_, _>>();
        for tunnel in tunnels {
            if let Some(link) = &mut tunnel.wss
                && let Some(title) = titles.get(&link.source)
            {
                link.title.clone_from(title);
            }
        }
        Ok(())
    }

    /// 创建独立 WSS 隧道，保留源 TCP 隧道的节点、本地地址和生命周期。
    pub fn create_wss(&self, source: u64) -> Result<CreatedTunnel, String> {
        let mut tunnels: Vec<Tunnel> = self.get("/tunnels")?;
        for tunnel in &mut tunnels {
            tunnel.load_wss_link();
        }
        let parent = tunnels
            .iter()
            .find(|tunnel| tunnel.id == source)
            .filter(|tunnel| tunnel.can_create_wss())
            .ok_or("源隧道不是普通 TCP 隧道，请刷新后重试")?;
        if let Some(existing) = tunnels
            .iter()
            .find(|tunnel| tunnel.wss_source() == Some(source))
        {
            // A repeated click must not consume another tunnel or domain record.
            return Ok(CreatedTunnel {
                id: existing.id,
                name: existing.name.clone(),
                remote: Some(existing.remote.clone()),
            });
        }
        let user = self.user_info()?;
        if user
            .tunnels
            .is_some_and(|limit| tunnels.len() >= limit as usize)
        {
            return Err("隧道名额已用完，创建 WSS 需要一个额外名额".to_owned());
        }
        let nodes: HashMap<String, Node> = self.get("/nodes")?;
        let mut node = nodes
            .get(&parent.node.to_string())
            .cloned()
            .ok_or("源隧道节点不可用")?;
        node.id = parent.node;
        if !node.can_create_for(&user) {
            return Err("当前账户无法在源节点创建 WSS 隧道".to_owned());
        }
        let domains = self.acquire_wss_domain()?;
        let rr = domains.available_record()?;
        let name = wss_name(parent, &tunnels);
        let request = CreateTunnel {
            name,
            kind: TunnelType::Tcp,
            node: parent.node,
            local_ip: parent.local_ip.clone(),
            local_port: parent.local_port,
            remote: None,
            public_game: false,
        };
        let note = WssLink {
            source,
            title: parent.name.clone(),
        }
        .note()?;
        let created = self.create_tunnel_with_extra(&request, &node, &note, WSS_EXTRA)?;
        if let Err(error) = self.bind_wss_domain(&domains, &rr, created.id) {
            return Err(match self.delete_tunnel(created.id) {
                Ok(false) => format!("WSS 域名绑定失败，新建隧道已撤销：{error}"),
                _ => format!(
                    "WSS 域名绑定失败，请手动删除未完成的隧道 {} 后重试：{error}",
                    created.id
                ),
            });
        }
        Ok(created)
    }
}

fn wss_name(parent: &Tunnel, tunnels: &[Tunnel]) -> String {
    let stem = parent
        .name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .take(16)
        .collect::<String>();
    let name = format!("{stem}_WSS");
    if !tunnels.iter().any(|tunnel| tunnel.name == name) {
        name
    } else {
        format!("WSS_{:016x}", parent.id)
    }
}
