use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize)]
pub(super) struct VersionManifest {
    pub versions: Vec<ManifestVersion>,
}

#[derive(Deserialize)]
pub(super) struct ManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: Option<String>,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Clone, Deserialize)]
pub(super) struct VersionDetail {
    pub id: String,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    #[serde(default)]
    pub downloads: HashMap<String, DownloadInfo>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    pub arguments: Option<LaunchArguments>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndexInfo>,
    pub assets: Option<String>,
    pub logging: Option<Logging>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
}

#[derive(Clone, Deserialize)]
pub(super) struct DownloadInfo {
    pub url: String,
    pub sha1: Option<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct AssetIndexInfo {
    pub id: String,
    pub url: String,
    pub sha1: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Deserialize)]
pub(super) struct AssetObject {
    pub hash: String,
}

#[derive(Clone, Deserialize)]
pub(super) struct Library {
    pub name: String,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    pub extract: Option<Extraction>,
}

#[derive(Clone, Deserialize)]
pub(super) struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<HashMap<String, LibraryArtifact>>,
}

#[derive(Clone, Deserialize)]
pub(super) struct LibraryArtifact {
    pub path: Option<String>,
    pub url: String,
    pub sha1: Option<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Clone, Deserialize)]
pub(super) struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct Extraction {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct LaunchArguments {
    #[serde(default)]
    pub game: Vec<Value>,
    #[serde(default)]
    pub jvm: Vec<Value>,
}

#[derive(Clone, Deserialize)]
pub(super) struct Logging {
    pub client: Option<LoggingClient>,
}

#[derive(Clone, Deserialize)]
pub(super) struct LoggingClient {
    pub argument: String,
    pub file: LoggingFile,
}

#[derive(Clone, Deserialize)]
pub(super) struct LoggingFile {
    pub id: String,
    pub url: String,
    pub sha1: Option<String>,
}

#[derive(Clone, Deserialize)]
pub(super) struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

pub(super) fn allowed(rules: Option<&[Rule]>) -> bool {
    let Some(rules) = rules else {
        return true;
    };
    let mut result = false;
    for rule in rules {
        if rule_matches(rule) {
            result = rule.action == "allow";
        }
    }
    result
}

fn rule_matches(rule: &Rule) -> bool {
    if let Some(features) = &rule.features
        && features.values().any(|required| *required)
    {
        return false;
    }
    let Some(os) = &rule.os else {
        return true;
    };
    if let Some(name) = &os.name
        && name != os_name()
    {
        return false;
    }
    if let Some(arch) = &os.arch {
        let current = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else {
            "aarch64"
        };
        if arch != current
            && arch
                != if cfg!(target_pointer_width = "64") {
                    "64"
                } else {
                    "32"
                }
        {
            return false;
        }
    }
    // Mojang uses Java regular expressions here. A platform-version restriction is rare;
    // reject it rather than accidentally including a binary for a different OS release.
    os.version.is_none()
}

pub(super) fn os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

pub(super) fn selected_native(library: &Library) -> Option<&LibraryArtifact> {
    let name = library.natives.as_ref()?.get(os_name())?;
    let arch = if cfg!(target_pointer_width = "64") {
        "64"
    } else {
        "32"
    };
    let classifier = name.replace("${arch}", arch);
    library
        .downloads
        .as_ref()?
        .classifiers
        .as_ref()?
        .get(&classifier)
}
