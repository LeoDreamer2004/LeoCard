use super::download::{DownloadFile, Downloader};
use super::loaders::LoaderKind;
use super::metadata::{
    AssetIndex, Library, ManifestVersion, VersionDetail, VersionManifest, allowed, selected_native,
};
use super::paths::{default_root, maven_path, safe_relative, safe_segment};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const VERSION_MANIFEST: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Clone, Debug)]
pub struct RemoteVersion {
    pub id: String,
    pub kind: String,
    pub release_time: String,
    pub installed: bool,
}

#[derive(Clone, Debug)]
pub struct InstalledVersion {
    pub id: String,
    pub loader: Option<LoaderKind>,
}

#[derive(Clone)]
pub struct MinecraftLauncher {
    pub(super) root: PathBuf,
    pub(super) downloader: Downloader,
}

#[derive(Deserialize)]
struct FabricLoader {
    loader: FabricLoaderVersion,
}

#[derive(Deserialize)]
struct FabricLoaderVersion {
    version: String,
    stable: bool,
}

impl MinecraftLauncher {
    pub fn new() -> Result<Self, String> {
        Self::at(default_root()?)
    }

    pub fn at(root: PathBuf) -> Result<Self, String> {
        Ok(Self {
            root,
            downloader: Downloader::new()?,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn versions(&self) -> Result<Vec<RemoteVersion>, String> {
        let manifest: VersionManifest = self.downloader.json(VERSION_MANIFEST)?;
        Ok(manifest
            .versions
            .into_iter()
            .filter(|version| safe_segment(&version.id).is_ok())
            .map(|version| RemoteVersion {
                installed: self.version_complete(&version.id).is_file(),
                id: version.id,
                kind: version.kind,
                release_time: version.release_time,
            })
            .collect())
    }

    pub fn installed_versions(&self) -> Result<Vec<InstalledVersion>, String> {
        let directory = self.root.join("versions");
        if !directory.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let id = entry.file_name().to_string_lossy().into_owned();
            if safe_segment(&id).is_err() || !self.version_complete(&id).is_file() {
                continue;
            }
            let detail = self.read_version(&id)?;
            versions.push(InstalledVersion {
                loader: LoaderKind::from_version_id(&id, detail.inherits_from.is_some()),
                id,
            });
        }
        versions.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(versions)
    }

    pub fn install_vanilla(
        &self,
        id: &str,
        mut progress: impl FnMut(String),
    ) -> Result<(), String> {
        safe_segment(id)?;
        progress(format!("获取 {id} 版本信息"));
        let manifest: VersionManifest = self.downloader.json(VERSION_MANIFEST)?;
        let version = manifest
            .versions
            .into_iter()
            .find(|version| version.id == id)
            .ok_or_else(|| format!("找不到 Minecraft 版本 {id}"))?;
        self.install_manifest_version(&version, &mut progress)
    }

    pub fn install_fabric(
        &self,
        game_version: &str,
        mut progress: impl FnMut(String),
    ) -> Result<String, String> {
        safe_segment(game_version)?;
        self.install_vanilla(game_version, &mut progress)?;
        progress("获取 Fabric 加载器".to_owned());
        let loaders: Vec<FabricLoader> = self.downloader.json(&format!(
            "https://meta.fabricmc.net/v2/versions/loader/{game_version}"
        ))?;
        let loader = loaders
            .iter()
            .find(|loader| loader.loader.stable)
            .or_else(|| loaders.first())
            .ok_or("该 Minecraft 版本没有可用的 Fabric 加载器")?;
        let loader_version = safe_segment(&loader.loader.version)?;
        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{game_version}/{loader_version}/profile/json"
        );
        let profile: serde_json::Value = self.downloader.json(&url)?;
        let id = profile
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or("Fabric 配置缺少版本名称")?;
        safe_segment(id)?;
        let detail: VersionDetail = serde_json::from_value(profile.clone())
            .map_err(|error| format!("Fabric 配置无效：{error}"))?;
        if detail.inherits_from.as_deref() != Some(game_version) {
            return Err("Fabric 配置对应的 Minecraft 版本不一致".to_owned());
        }
        let destination = self.version_json(id);
        fs::create_dir_all(destination.parent().ok_or("无效的 Fabric 目录")?)
            .map_err(|error| error.to_string())?;
        fs::write(
            &destination,
            serde_json::to_vec_pretty(&profile).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        self.install_detail(&detail, &mut progress)?;
        Ok(id.to_owned())
    }

    pub fn game_version(&self, id: &str) -> Result<String, String> {
        let detail = self.read_version(id)?;
        Ok(detail.inherits_from.unwrap_or(detail.id))
    }

    fn install_manifest_version(
        &self,
        version: &ManifestVersion,
        progress: &mut impl FnMut(String),
    ) -> Result<(), String> {
        let metadata_file = DownloadFile {
            url: version.url.clone(),
            target: self.version_json(&version.id),
            sha1: version.sha1.clone(),
        };
        self.downloader.file(&metadata_file)?;
        let detail = self.read_version(&version.id)?;
        if detail.id != version.id {
            return Err("下载的版本信息与所选版本不一致".to_owned());
        }
        self.install_detail(&detail, progress)
    }

    pub(super) fn install_detail(
        &self,
        detail: &VersionDetail,
        progress: &mut impl FnMut(String),
    ) -> Result<(), String> {
        let mut files = self.library_files(&detail.libraries)?;
        if let Some(client) = detail.downloads.get("client") {
            files.push(DownloadFile {
                url: client.url.clone(),
                target: self.version_jar(&detail.id),
                sha1: client.sha1.clone(),
            });
        }
        if let Some(logging) = detail
            .logging
            .as_ref()
            .and_then(|logging| logging.client.as_ref())
        {
            files.push(DownloadFile {
                url: logging.file.url.clone(),
                target: self
                    .root
                    .join("assets/log_configs")
                    .join(safe_segment(&logging.file.id)?),
                sha1: logging.file.sha1.clone(),
            });
        }
        progress("下载游戏文件和依赖库".to_owned());
        self.download_files(files, progress)?;
        if let Some(index) = &detail.asset_index {
            progress("获取资源索引".to_owned());
            let index_path = self
                .root
                .join("assets/indexes")
                .join(format!("{}.json", safe_segment(&index.id)?));
            self.downloader.file(&DownloadFile {
                url: index.url.clone(),
                target: index_path.clone(),
                sha1: index.sha1.clone(),
            })?;
            let data = fs::read(index_path).map_err(|error| error.to_string())?;
            let index: AssetIndex =
                serde_json::from_slice(&data).map_err(|error| error.to_string())?;
            let mut objects = Vec::with_capacity(index.objects.len());
            let mut seen = HashSet::new();
            for object in index.objects.into_values() {
                let hash = object.hash;
                if hash.len() != 40 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    return Err("资源索引中包含无效的哈希值".to_owned());
                }
                if !seen.insert(hash.clone()) {
                    continue;
                }
                let prefix = &hash[..2];
                objects.push(DownloadFile {
                    url: format!("https://resources.download.minecraft.net/{prefix}/{hash}"),
                    target: self.root.join("assets/objects").join(prefix).join(&hash),
                    sha1: Some(hash),
                });
            }
            progress("下载游戏资源".to_owned());
            self.download_files(objects, progress)?;
        }
        fs::write(self.version_complete(&detail.id), b"complete")
            .map_err(|error| format!("无法标记安装完成：{error}"))?;
        Ok(())
    }

    pub(super) fn library_files(&self, libraries: &[Library]) -> Result<Vec<DownloadFile>, String> {
        let mut files = Vec::new();
        for library in libraries
            .iter()
            .filter(|library| allowed(library.rules.as_deref()))
        {
            if let Some(artifact) = library
                .downloads
                .as_ref()
                .and_then(|downloads| downloads.artifact.as_ref())
            {
                let path = match &artifact.path {
                    Some(path) => safe_relative(path)?,
                    None => maven_path(&library.name)?,
                };
                files.push(DownloadFile {
                    url: artifact.url.clone(),
                    target: self.root.join("libraries").join(path),
                    sha1: artifact.sha1.clone(),
                });
            } else if library.natives.is_none() {
                let path = maven_path(&library.name)?;
                let base = library
                    .url
                    .as_deref()
                    .unwrap_or("https://libraries.minecraft.net/");
                files.push(DownloadFile {
                    url: format!(
                        "{}/{}",
                        base.trim_end_matches('/'),
                        path.to_string_lossy().replace('\\', "/")
                    ),
                    target: self.root.join("libraries").join(path),
                    sha1: library.sha1.clone(),
                });
            }
            if let Some(native) = selected_native(library) {
                let path = native.path.as_ref().ok_or("原生库缺少下载路径")?;
                files.push(DownloadFile {
                    url: native.url.clone(),
                    target: self.root.join("libraries").join(safe_relative(path)?),
                    sha1: native.sha1.clone(),
                });
            }
        }
        Ok(files)
    }

    fn download_files(
        &self,
        files: Vec<DownloadFile>,
        progress: &mut impl FnMut(String),
    ) -> Result<(), String> {
        let mut unique = HashSet::new();
        let files = files
            .into_iter()
            .filter(|file| unique.insert(file.target.clone()))
            .collect();
        self.downloader.files(files, |done, total| {
            progress(format!("下载文件 {done}/{total}"));
        })
    }

    pub(super) fn read_version(&self, id: &str) -> Result<VersionDetail, String> {
        safe_segment(id)?;
        let data = fs::read(self.version_json(id))
            .map_err(|error| format!("读取 Minecraft 版本 {id} 失败：{error}"))?;
        serde_json::from_slice(&data)
            .map_err(|error| format!("解析 Minecraft 版本 {id} 失败：{error}"))
    }

    pub(super) fn version_json(&self, id: &str) -> PathBuf {
        self.root
            .join("versions")
            .join(id)
            .join(format!("{id}.json"))
    }

    pub(super) fn version_jar(&self, id: &str) -> PathBuf {
        self.root
            .join("versions")
            .join(id)
            .join(format!("{id}.jar"))
    }

    pub(super) fn version_complete(&self, id: &str) -> PathBuf {
        self.root
            .join("versions")
            .join(id)
            .join(".leocard-complete")
    }

    pub(super) fn instance_dir(&self, id: &str) -> PathBuf {
        self.root.join("instances").join(id)
    }
}
