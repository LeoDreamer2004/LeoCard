use super::download::DownloadFile;
use super::install::MinecraftLauncher;
use super::paths::safe_segment;
use std::cmp::Ordering;
use std::fs;
use std::process::Command;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoaderKind {
    Fabric,
    Forge,
    NeoForge,
}

impl LoaderKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::Forge => "Forge",
            Self::NeoForge => "NeoForge",
        }
    }

    pub fn modrinth_name(self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Forge => "forge",
            Self::NeoForge => "neoforge",
        }
    }

    pub fn curseforge_type(self) -> u8 {
        match self {
            Self::Forge => 1,
            Self::Fabric => 4,
            Self::NeoForge => 6,
        }
    }

    pub fn from_version_id(id: &str, inherited: bool) -> Option<Self> {
        if !inherited {
            return None;
        }
        if id.contains("neoforge") {
            Some(Self::NeoForge)
        } else if id.contains("forge") {
            Some(Self::Forge)
        } else if id.contains("fabric") {
            Some(Self::Fabric)
        } else {
            None
        }
    }
}

impl MinecraftLauncher {
    pub fn install_forge(
        &self,
        game_version: &str,
        progress: impl FnMut(String),
    ) -> Result<String, String> {
        self.install_jar_loader(LoaderKind::Forge, game_version, progress)
    }

    pub fn install_neoforge(
        &self,
        game_version: &str,
        progress: impl FnMut(String),
    ) -> Result<String, String> {
        self.install_jar_loader(LoaderKind::NeoForge, game_version, progress)
    }

    fn install_jar_loader(
        &self,
        kind: LoaderKind,
        game_version: &str,
        mut progress: impl FnMut(String),
    ) -> Result<String, String> {
        safe_segment(game_version)?;
        self.install_vanilla(game_version, &mut progress)?;
        progress(format!("获取 {} 加载器版本", kind.label()));
        let (metadata, prefix, artifact) = match kind {
            LoaderKind::Forge => (
                "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml",
                format!("{game_version}-"),
                "forge",
            ),
            LoaderKind::NeoForge => (
                "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml",
                format!(
                    "{}.",
                    game_version
                        .strip_prefix("1.")
                        .ok_or("NeoForge 不支持此游戏版本")?
                ),
                "neoforge",
            ),
            LoaderKind::Fabric => return Err("Fabric 使用独立安装流程".to_owned()),
        };
        let xml = self.downloader.text(metadata)?;
        let version = xml
            .split("<version>")
            .filter_map(|part| part.split_once("</version>").map(|(value, _)| value))
            .filter(|version| version.starts_with(&prefix))
            .filter(|version| safe_segment(version).is_ok())
            .max_by(|left, right| compare_versions(left, right))
            .ok_or_else(|| {
                format!(
                    "{} 没有适用于 Minecraft {game_version} 的版本",
                    kind.label()
                )
            })?;
        let base = if kind == LoaderKind::Forge {
            "https://maven.minecraftforge.net/net/minecraftforge/forge"
        } else {
            "https://maven.neoforged.net/releases/net/neoforged/neoforge"
        };
        let installer = self
            .root
            .join("installers")
            .join(format!("{artifact}-{version}-installer.jar"));
        let installer_url = format!("{base}/{version}/{artifact}-{version}-installer.jar");
        let checksum = self.downloader.text(&format!("{installer_url}.sha1"))?;
        let checksum = checksum
            .split_whitespace()
            .next()
            .ok_or("安装器缺少 SHA-1 校验值")?;
        if checksum.len() != 40 || !checksum.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("安装器 SHA-1 校验值无效".to_owned());
        }
        self.downloader.file(&DownloadFile {
            url: installer_url,
            target: installer.clone(),
            sha1: Some(checksum.to_owned()),
        })?;
        progress(format!("安装 {} {version}", kind.label()));
        fs::create_dir_all(&self.root).map_err(|error| error.to_string())?;
        let output = Command::new(super::launch::java_executable())
            .arg("-jar")
            .arg(&installer)
            .arg("--installClient")
            .arg(&self.root)
            .current_dir(&self.root)
            .output()
            .map_err(|error| format!("运行 {} 安装器失败：{error}", kind.label()))?;
        if !output.status.success() {
            let details = if output.stderr.is_empty() {
                String::from_utf8_lossy(&output.stdout)
            } else {
                String::from_utf8_lossy(&output.stderr)
            };
            return Err(format!("{} 安装器失败：{}", kind.label(), details.trim()));
        }
        let directory = self.root.join("versions");
        let id = fs::read_dir(directory)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|id| id.contains(kind.modrinth_name()) && safe_segment(id).is_ok())
            .filter(|id| self.version_json(id).is_file())
            .find(|id| {
                let loader_version = version.split_once('-').map_or(version, |(_, value)| value);
                self.read_version(id).is_ok_and(|detail| {
                    detail.inherits_from.as_deref() == Some(game_version)
                        && id.contains(loader_version)
                })
            })
            .ok_or_else(|| format!("{} 安装器未生成游戏版本", kind.label()))?;
        let detail = self.read_version(&id)?;
        if detail.id != id {
            return Err(format!("{} 安装器生成的版本名称不一致", kind.label()));
        }
        safe_segment(&detail.id)?;
        self.install_detail(&detail, &mut progress)?;
        Ok(id)
    }
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    left.split(['.', '-'])
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .cmp(
            right
                .split(['.', '-'])
                .map(|part| part.parse::<u32>().unwrap_or(0)),
        )
}
