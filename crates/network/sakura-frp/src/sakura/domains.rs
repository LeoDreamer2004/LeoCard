use super::api::SakuraClient;
use super::tunnel::Tunnel;
use serde::Deserialize;
use serde::de::IgnoredAny;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize)]
pub(super) struct DomainSettings {
    pub domain: Option<String>,
    #[serde(default)]
    pub records: Vec<DomainRecord>,
    #[serde(default)]
    pub words: Vec<String>,
    #[serde(default)]
    pub acquire: HashMap<String, IgnoredAny>,
}

#[derive(Deserialize)]
pub(super) struct DomainRecord {
    pub rr: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub tunnel: u64,
}

impl DomainSettings {
    pub(super) fn assign(&self, tunnels: &mut [Tunnel]) {
        let Some(domain) = &self.domain else { return };
        for record in self.records.iter().filter(|record| record.kind == "CNAME") {
            if let Some(tunnel) = tunnels.iter_mut().find(|tunnel| tunnel.id == record.tunnel) {
                tunnel.bound_domain = Some(if record.rr == "@" {
                    domain.clone()
                } else {
                    format!("{}.{domain}", record.rr)
                });
            }
        }
    }

    pub(super) fn available_record(&self) -> Result<String, String> {
        let occupied = self
            .records
            .iter()
            .filter(|record| record.kind == "CNAME")
            .map(|record| record.rr.as_str())
            .collect::<HashSet<_>>();
        let mut words = self
            .words
            .iter()
            .map(String::as_str)
            .filter(|word| *word != "@")
            .collect::<Vec<_>>();
        words.sort_by_key(|word| match *word {
            "game" => 0,
            "server" => 1,
            "www" => 2,
            _ => 3,
        });
        for first in &words {
            if !occupied.contains(first) {
                return Ok((*first).to_owned());
            }
        }
        for first in &words {
            for second in &words {
                let name = format!("{first}-{second}");
                if !occupied.contains(name.as_str()) {
                    return Ok(name);
                }
            }
        }
        for first in &words {
            for second in &words {
                for third in &words {
                    let name = format!("{first}-{second}-{third}");
                    if !occupied.contains(name.as_str()) {
                        return Ok(name);
                    }
                }
            }
        }
        if self.words.iter().any(|word| word == "@") && !occupied.contains("@") {
            return Ok("@".to_owned());
        }
        Err("没有可用于 WSS 的子域名，请先在 SakuraFrp 管理面板整理子域绑定".to_owned())
    }
}

impl SakuraClient {
    pub(super) fn domain_settings(&self) -> Result<DomainSettings, String> {
        self.get("/domain/get")
    }

    pub(super) fn acquire_wss_domain(&self) -> Result<DomainSettings, String> {
        let settings = self.domain_settings()?;
        if settings.domain.is_some() {
            return Ok(settings);
        }
        let style = ["hash", "random", "uid", "randnum"]
            .into_iter()
            .find(|style| settings.acquire.contains_key(*style))
            .ok_or("账户暂时无法申请免费子域名，请先在 SakuraFrp 完成所需账户配置")?;
        let _: DomainSettings = self.json(
            self.authorized(self.http.post(self.url("/domain/acquire")))
                .form(&[("type", style)]),
        )?;
        self.domain_settings()
    }

    pub(super) fn bind_wss_domain(
        &self,
        settings: &DomainSettings,
        rr: &str,
        tunnel: u64,
    ) -> Result<(), String> {
        let domain = settings.domain.as_deref().ok_or("账户尚未分配免费子域名")?;
        self.empty(
            self.authorized(self.http.post(self.url("/domain/bind")))
                .form(&[
                    ("rr", rr.to_owned()),
                    ("tunnel", tunnel.to_string()),
                    ("domain", domain.to_owned()),
                    ("type", "CNAME".to_owned()),
                ]),
        )
    }
}
