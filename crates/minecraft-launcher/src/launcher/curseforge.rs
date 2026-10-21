use super::download::DownloadFile;
use super::install::MinecraftLauncher;
use super::loaders::LoaderKind;
use super::modrinth::{ResourceKind, ResourceProject};
use super::paths::safe_segment;
use serde::Deserialize;
use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs;
use std::path::PathBuf;

const API: &str = "https://api.curseforge.com/v1";

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseProject {
    id: u64,
    name: String,
    summary: String,
    download_count: f64,
    logo: Option<CurseLogo>,
}

#[derive(Deserialize)]
struct CurseLogo {
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseFile {
    id: u64,
    file_name: String,
    download_url: Option<String>,
    #[serde(default)]
    dependencies: Vec<CurseDependency>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseDependency {
    mod_id: u64,
    relation_type: u8,
}

impl MinecraftLauncher {
    pub(super) fn search_curseforge(
        &self,
        kind: ResourceKind,
        query: &str,
        game_version: &str,
        loader: Option<LoaderKind>,
    ) -> Result<Vec<ResourceProject>, String> {
        safe_segment(game_version)?;
        let key = api_key()?;
        let mut url = format!(
            "{API}/mods/search?gameId=432&classId={}&pageSize=30&gameVersion={game_version}",
            class_id(kind)
        );
        if !query.trim().is_empty() {
            url.push_str("&searchFilter=");
            url.push_str(&encode_query(query));
        }
        if kind == ResourceKind::Mod {
            url.push_str(&format!(
                "&modLoaderType={}",
                loader.ok_or("请先选择加载器版本")?.curseforge_type()
            ));
        }
        let result: Envelope<Vec<CurseProject>> = self.downloader.json_with_key(&url, &key)?;
        Ok(result
            .data
            .into_iter()
            .map(|project| ResourceProject {
                project_id: project.id.to_string(),
                title: project.name,
                description: project.summary,
                downloads: project.download_count as u64,
                icon_url: project.logo.map(|logo| logo.url),
            })
            .collect())
    }

    pub(super) fn install_curseforge(
        &self,
        kind: ResourceKind,
        project_id: &str,
        version_id: &str,
    ) -> Result<PathBuf, String> {
        let key = api_key()?;
        let project_id = project_id
            .parse::<u64>()
            .map_err(|_| "无效的 CurseForge 项目 ID")?;
        safe_segment(version_id)?;
        let detail = self.read_version(version_id)?;
        let game_version = detail.inherits_from.as_deref().unwrap_or(version_id);
        let loader = LoaderKind::from_version_id(version_id, detail.inherits_from.is_some());
        if kind == ResourceKind::Mod && loader.is_none() {
            return Err("安装 Mod 前请先选择加载器版本".to_owned());
        }
        let file_loader = if kind == ResourceKind::Mod {
            loader
        } else {
            None
        };
        let selected = self.curse_file(project_id, game_version, file_loader, &key)?;
        let destination = self.download_curse_file(kind, project_id, version_id, &selected)?;
        if kind == ResourceKind::Mod {
            let mut pending = VecDeque::from(selected.dependencies);
            let mut visited = HashSet::from([project_id]);
            while let Some(dependency) = pending.pop_front() {
                if dependency.relation_type != 3 || !visited.insert(dependency.mod_id) {
                    continue;
                }
                let file = self.curse_file(dependency.mod_id, game_version, file_loader, &key)?;
                self.download_curse_file(kind, dependency.mod_id, version_id, &file)?;
                pending.extend(file.dependencies);
            }
        }
        Ok(destination)
    }

    fn curse_file(
        &self,
        project_id: u64,
        game_version: &str,
        loader: Option<LoaderKind>,
        key: &str,
    ) -> Result<CurseFile, String> {
        let mut url =
            format!("{API}/mods/{project_id}/files?gameVersion={game_version}&pageSize=50");
        if let Some(loader) = loader {
            url.push_str(&format!("&modLoaderType={}", loader.curseforge_type()));
        }
        let files: Envelope<Vec<CurseFile>> = self.downloader.json_with_key(&url, key)?;
        files
            .data
            .into_iter()
            .find(|file| file.download_url.is_some())
            .ok_or_else(|| format!("CurseForge 项目 {project_id} 没有可直接下载的兼容文件"))
    }

    fn download_curse_file(
        &self,
        kind: ResourceKind,
        project_id: u64,
        version_id: &str,
        file: &CurseFile,
    ) -> Result<PathBuf, String> {
        let extension = file
            .file_name
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase())
            .ok_or("资源文件缺少扩展名")?;
        if (kind == ResourceKind::Mod && extension != "jar")
            || (kind != ResourceKind::Mod && extension != "zip" && extension != "jar")
        {
            return Err("资源文件格式不受支持".to_owned());
        }
        let destination = self
            .instance_dir(version_id)
            .join(kind.directory())
            .join(format!("{project_id}-{}.{}", file.id, extension));
        fs::create_dir_all(destination.parent().ok_or("无效的资源目录")?)
            .map_err(|error| error.to_string())?;
        self.downloader.file(&DownloadFile {
            url: file
                .download_url
                .clone()
                .ok_or("作者禁止第三方下载此文件")?,
            target: destination.clone(),
            sha1: None,
        })?;
        Ok(destination)
    }
}

fn api_key() -> Result<String, String> {
    env::var("LEOCARD_CURSEFORGE_API_KEY")
        .ok()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| "请先设置 LEOCARD_CURSEFORGE_API_KEY（CurseForge 官方 API 密钥）".to_owned())
}

fn class_id(kind: ResourceKind) -> u32 {
    match kind {
        ResourceKind::Mod => 6,
        ResourceKind::ResourcePack => 12,
        ResourceKind::Shader => 6552,
        ResourceKind::DataPack => 6945,
    }
}

fn encode_query(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
