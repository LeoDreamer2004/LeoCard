use super::download::DownloadFile;
use super::install::MinecraftLauncher;
use super::loaders::LoaderKind;
use super::paths::safe_segment;
use serde::Deserialize;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;

const API: &str = "https://api.modrinth.com/v2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Mod,
    ResourcePack,
    Shader,
    DataPack,
}

impl ResourceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Mod => "Mod",
            Self::ResourcePack => "资源包",
            Self::Shader => "光影",
            Self::DataPack => "数据包",
        }
    }

    fn project_type(self) -> &'static str {
        match self {
            Self::Mod => "mod",
            Self::ResourcePack => "resourcepack",
            Self::Shader => "shader",
            Self::DataPack => "datapack",
        }
    }

    pub(super) fn directory(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resourcepacks",
            Self::Shader => "shaderpacks",
            Self::DataPack => "datapacks",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ResourceProject {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceSource {
    Modrinth,
    CurseForge,
}

impl ResourceSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Modrinth => "Modrinth",
            Self::CurseForge => "CurseForge",
        }
    }
}

#[derive(Deserialize)]
struct SearchResults {
    hits: Vec<ResourceProject>,
}

#[derive(Deserialize)]
struct ProjectVersion {
    id: String,
    files: Vec<ProjectFile>,
    #[serde(default)]
    dependencies: Vec<ProjectDependency>,
}

#[derive(Deserialize)]
struct ProjectDependency {
    project_id: Option<String>,
    version_id: Option<String>,
    dependency_type: String,
}

#[derive(Deserialize)]
struct ProjectFile {
    url: String,
    filename: String,
    primary: Option<bool>,
    hashes: FileHashes,
}

#[derive(Deserialize)]
struct FileHashes {
    sha1: Option<String>,
}

impl MinecraftLauncher {
    pub fn search_resources(
        &self,
        source: ResourceSource,
        kind: ResourceKind,
        query: &str,
        game_version: &str,
        loader: Option<LoaderKind>,
    ) -> Result<Vec<ResourceProject>, String> {
        if source == ResourceSource::CurseForge {
            return self.search_curseforge(kind, query, game_version, loader);
        }
        safe_segment(game_version)?;
        let mut facets = vec![
            vec![format!("project_type:{}", kind.project_type())],
            vec![format!("versions:{game_version}")],
        ];
        if kind == ResourceKind::Mod {
            let loader = loader.ok_or("安装 Mod 前请先选择加载器版本")?;
            facets.push(vec![format!("categories:{}", loader.modrinth_name())]);
        }
        let facets = serde_json::to_string(&facets).map_err(|error| error.to_string())?;
        let results: SearchResults = self.downloader.json_query(
            &format!("{API}/search"),
            &[("query", query), ("facets", &facets), ("limit", "30")],
        )?;
        Ok(results.hits)
    }

    pub fn install_resource(
        &self,
        source: ResourceSource,
        kind: ResourceKind,
        project_id: &str,
        version_id: &str,
    ) -> Result<PathBuf, String> {
        if source == ResourceSource::CurseForge {
            return self.install_curseforge(kind, project_id, version_id);
        }
        safe_segment(project_id)?;
        safe_segment(version_id)?;
        let detail = self.read_version(version_id)?;
        let loader = LoaderKind::from_version_id(version_id, detail.inherits_from.is_some());
        if kind == ResourceKind::Mod && loader.is_none() {
            return Err("安装 Mod 前请先安装并选择加载器版本".to_owned());
        }
        let game_version = detail.inherits_from.as_deref().unwrap_or(version_id);
        let game_versions = serde_json::json!([game_version]).to_string();
        let loaders =
            serde_json::json!([loader.map(LoaderKind::modrinth_name).unwrap_or_default()])
                .to_string();
        let mut query = vec![("game_versions", game_versions.as_str())];
        if kind == ResourceKind::Mod {
            query.push(("loaders", loaders.as_str()));
        }
        let selected = self.compatible_version(project_id, &query, game_version)?;
        let destination = self.download_resource_file(kind, project_id, version_id, &selected)?;
        if kind == ResourceKind::Mod {
            self.install_mod_dependencies(selected.dependencies, version_id, &query, game_version)?;
        }
        Ok(destination)
    }

    fn compatible_version(
        &self,
        project_id: &str,
        query: &[(&str, &str)],
        game_version: &str,
    ) -> Result<ProjectVersion, String> {
        let versions: Vec<ProjectVersion> = self
            .downloader
            .json_query(&format!("{API}/project/{project_id}/version"), query)?;
        versions
            .into_iter()
            .find(|version| !version.files.is_empty())
            .ok_or_else(|| format!("该资源没有适用于 Minecraft {game_version} 的下载文件"))
    }

    fn install_mod_dependencies(
        &self,
        dependencies: Vec<ProjectDependency>,
        version_id: &str,
        query: &[(&str, &str)],
        game_version: &str,
    ) -> Result<(), String> {
        let mut pending = VecDeque::from(dependencies);
        let mut visited = HashSet::new();
        while let Some(dependency) = pending.pop_front() {
            if dependency.dependency_type != "required" {
                continue;
            }
            let Some(project_id) = dependency.project_id else {
                continue;
            };
            safe_segment(&project_id)?;
            if !visited.insert(project_id.clone()) {
                continue;
            }
            let selected = if let Some(dependency_version) = dependency.version_id {
                safe_segment(&dependency_version)?;
                self.downloader
                    .json(&format!("{API}/version/{dependency_version}"))?
            } else {
                self.compatible_version(&project_id, query, game_version)?
            };
            self.download_resource_file(ResourceKind::Mod, &project_id, version_id, &selected)?;
            pending.extend(selected.dependencies);
        }
        Ok(())
    }

    fn download_resource_file(
        &self,
        kind: ResourceKind,
        project_id: &str,
        version_id: &str,
        selected: &ProjectVersion,
    ) -> Result<PathBuf, String> {
        safe_segment(&selected.id)?;
        let file = selected
            .files
            .iter()
            .find(|file| file.primary == Some(true))
            .or_else(|| selected.files.first())
            .ok_or("资源版本没有文件")?;
        let extension = file
            .filename
            .rsplit_once('.')
            .map(|(_, extension)| extension.to_ascii_lowercase())
            .ok_or("资源文件缺少扩展名")?;
        let valid_extension = match kind {
            ResourceKind::Mod => extension == "jar",
            _ => extension == "zip" || extension == "jar",
        };
        if !valid_extension {
            return Err("资源文件的格式不受支持".to_owned());
        }
        let destination = self
            .instance_dir(version_id)
            .join(kind.directory())
            .join(format!("{project_id}-{}.{}", selected.id, extension));
        fs::create_dir_all(destination.parent().ok_or("无效的资源目录")?)
            .map_err(|error| error.to_string())?;
        self.downloader.file(&DownloadFile {
            url: file.url.clone(),
            target: destination.clone(),
            sha1: file.hashes.sha1.clone(),
        })?;
        Ok(destination)
    }
}
