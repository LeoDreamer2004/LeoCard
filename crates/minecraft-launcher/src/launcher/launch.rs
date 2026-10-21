use super::auth::MinecraftAccount;
use super::install::MinecraftLauncher;
use super::metadata::{Library, Rule, VersionDetail, allowed, selected_native};
use super::paths::{maven_path, safe_relative, safe_segment};
use md5::{Digest, Md5};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use zip::ZipArchive;

impl MinecraftLauncher {
    pub fn launch_offline(&self, id: &str, username: &str) -> Result<Child, String> {
        safe_segment(id)?;
        if !(3..=16).contains(&username.len())
            || !username
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err("离线玩家名只能使用 3 到 16 位英文字母、数字或下划线".to_owned());
        }
        self.launch_with_identity(id, username, offline_uuid(username), "0", "0", "legacy")
    }

    pub fn launch_authenticated(
        &self,
        id: &str,
        account: &MinecraftAccount,
    ) -> Result<Child, String> {
        if account.access_token.is_empty() || account.uuid.len() != 32 || account.expired() {
            return Err("微软账号会话已失效，请退出并重新登录".to_owned());
        }
        self.launch_with_identity(
            id,
            &account.name,
            account.uuid.clone(),
            &account.access_token,
            &account.xuid,
            "msa",
        )
    }

    fn launch_with_identity(
        &self,
        id: &str,
        username: &str,
        uuid: String,
        access_token: &str,
        xuid: &str,
        user_type: &str,
    ) -> Result<Child, String> {
        safe_segment(id)?;
        let selected = self.read_version(id)?;
        let base = if let Some(parent) = &selected.inherits_from {
            safe_segment(parent)?;
            self.read_version(parent)?
        } else {
            selected.clone()
        };
        let libraries = selected
            .inherits_from
            .as_ref()
            .into_iter()
            .flat_map(|_| selected.libraries.iter())
            .chain(base.libraries.iter())
            .collect::<Vec<_>>();
        let game_dir = self.instance_dir(id);
        let natives_dir = game_dir.join("natives");
        fs::create_dir_all(&natives_dir).map_err(|error| error.to_string())?;
        self.extract_natives(&libraries, &natives_dir)?;
        let classpath = self.classpath(&base, &libraries)?;
        let asset_id = base
            .asset_index
            .as_ref()
            .map(|index| index.id.as_str())
            .or(base.assets.as_deref())
            .unwrap_or("legacy");
        let assets_root = self.root.join("assets");
        let replacements = HashMap::from([
            ("${auth_player_name}", username.to_owned()),
            ("${auth_uuid}", uuid),
            ("${auth_access_token}", access_token.to_owned()),
            ("${auth_session}", access_token.to_owned()),
            ("${auth_xuid}", xuid.to_owned()),
            ("${user_type}", user_type.to_owned()),
            ("${user_properties}", "{}".to_owned()),
            ("${version_name}", id.to_owned()),
            ("${version_type}", "release".to_owned()),
            ("${game_directory}", game_dir.to_string_lossy().into_owned()),
            ("${assets_root}", assets_root.to_string_lossy().into_owned()),
            (
                "${game_assets}",
                assets_root
                    .join("virtual/legacy")
                    .to_string_lossy()
                    .into_owned(),
            ),
            ("${assets_index_name}", asset_id.to_owned()),
            (
                "${natives_directory}",
                natives_dir.to_string_lossy().into_owned(),
            ),
            (
                "${library_directory}",
                self.root.join("libraries").to_string_lossy().into_owned(),
            ),
            ("${classpath}", classpath.clone()),
            ("${classpath_separator}", classpath_separator().to_owned()),
            ("${launcher_name}", "LeoCard".to_owned()),
            ("${launcher_version}", env!("CARGO_PKG_VERSION").to_owned()),
        ]);
        let mut command = Command::new(java_executable());
        command.current_dir(&game_dir);
        let log = File::create(game_dir.join("launcher.log"))
            .map_err(|error| format!("无法创建启动日志：{error}"))?;
        let errors = log.try_clone().map_err(|error| error.to_string())?;
        command.stdout(Stdio::from(log)).stderr(Stdio::from(errors));
        let mut jvm_args = base
            .arguments
            .as_ref()
            .map(|arguments| argument_values(&arguments.jvm))
            .unwrap_or_default();
        if selected.inherits_from.is_some() {
            jvm_args.extend(
                selected
                    .arguments
                    .as_ref()
                    .map(|arguments| argument_values(&arguments.jvm))
                    .unwrap_or_default(),
            );
        }
        if !jvm_args
            .iter()
            .any(|argument| argument.contains("${classpath}"))
        {
            jvm_args.extend(["-cp".to_owned(), "${classpath}".to_owned()]);
        }
        if !jvm_args
            .iter()
            .any(|argument| argument.contains("${natives_directory}"))
        {
            jvm_args.push("-Djava.library.path=${natives_directory}".to_owned());
        }
        jvm_args.push("-Dminecraft.launcher.brand=LeoCard".to_owned());
        if let Some(logging) = base
            .logging
            .as_ref()
            .and_then(|logging| logging.client.as_ref())
        {
            let path = self
                .root
                .join("assets/log_configs")
                .join(safe_segment(&logging.file.id)?);
            jvm_args.push(logging.argument.replace("${path}", &path.to_string_lossy()));
        }
        command.args(
            jvm_args
                .iter()
                .map(|argument| substitute(argument, &replacements)),
        );
        let main_class = selected
            .main_class
            .as_ref()
            .or(base.main_class.as_ref())
            .ok_or("版本信息缺少主类")?;
        command.arg(main_class);
        let game_args = selected
            .arguments
            .as_ref()
            .and_then(|arguments| (!arguments.game.is_empty()).then_some(arguments.game.as_slice()))
            .or_else(|| {
                base.arguments
                    .as_ref()
                    .map(|arguments| arguments.game.as_slice())
            })
            .map(argument_values)
            .or_else(|| {
                selected
                    .minecraft_arguments
                    .as_ref()
                    .or(base.minecraft_arguments.as_ref())
                    .map(|arguments| arguments.split_whitespace().map(str::to_owned).collect())
            })
            .unwrap_or_default();
        command.args(
            game_args
                .iter()
                .map(|argument| substitute(argument, &replacements)),
        );
        command.spawn().map_err(|error| {
            let major = base
                .java_version
                .as_ref()
                .map(|version| version.major_version);
            format!(
                "启动 Minecraft 失败：{error}。请安装 Java {}，或设置 LEOCARD_JAVA_PATH",
                major.map_or("运行时".to_owned(), |version| version.to_string())
            )
        })
    }

    fn classpath(&self, base: &VersionDetail, libraries: &[&Library]) -> Result<String, String> {
        let mut entries = Vec::new();
        let mut seen = HashSet::new();
        for library in libraries
            .iter()
            .copied()
            .filter(|library| allowed(library.rules.as_deref()))
        {
            if library.natives.is_some()
                && library
                    .downloads
                    .as_ref()
                    .and_then(|downloads| downloads.artifact.as_ref())
                    .is_none()
            {
                continue;
            }
            let identity = library
                .name
                .split(':')
                .take(2)
                .collect::<Vec<_>>()
                .join(":");
            if !seen.insert(identity) {
                continue;
            }
            let path = library
                .downloads
                .as_ref()
                .and_then(|downloads| downloads.artifact.as_ref())
                .and_then(|artifact| artifact.path.as_deref())
                .map(safe_relative)
                .transpose()?
                .unwrap_or(maven_path(&library.name)?);
            let file = self.root.join("libraries").join(path);
            if !file.is_file() {
                return Err(format!("缺少依赖库 {}，请重新安装版本", file.display()));
            }
            entries.push(file);
        }
        let jar = self.version_jar(&base.id);
        if !jar.is_file() {
            return Err(format!("缺少游戏文件 {}，请重新安装版本", jar.display()));
        }
        entries.push(jar);
        Ok(env::join_paths(&entries)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .into_owned())
    }

    fn extract_natives(&self, libraries: &[&Library], directory: &Path) -> Result<(), String> {
        for library in libraries
            .iter()
            .copied()
            .filter(|library| allowed(library.rules.as_deref()))
        {
            let Some(native) = selected_native(library) else {
                continue;
            };
            let path = native.path.as_deref().ok_or("原生库缺少路径")?;
            let archive_path = self.root.join("libraries").join(safe_relative(path)?);
            let archive = File::open(&archive_path)
                .map_err(|error| format!("无法打开 {}：{error}", archive_path.display()))?;
            let mut archive = ZipArchive::new(archive).map_err(|error| error.to_string())?;
            for index in 0..archive.len() {
                let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
                let Some(path) = entry.enclosed_name() else {
                    continue;
                };
                let name = path.to_string_lossy();
                if name.starts_with("META-INF/")
                    || library.extract.as_ref().is_some_and(|extract| {
                        extract
                            .exclude
                            .iter()
                            .any(|prefix| name.starts_with(prefix))
                    })
                    || entry.is_dir()
                {
                    continue;
                }
                let target = directory.join(path);
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                let mut output = File::create(target).map_err(|error| error.to_string())?;
                io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }
}

fn argument_values(values: &[Value]) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        if let Some(value) = value.as_str() {
            result.push(value.to_owned());
        } else if let Some(object) = value.as_object() {
            let rules: Option<Vec<Rule>> = object
                .get("rules")
                .and_then(|rules| serde_json::from_value(rules.clone()).ok());
            if !allowed(rules.as_deref()) {
                continue;
            }
            match object.get("value") {
                Some(Value::String(value)) => result.push(value.clone()),
                Some(Value::Array(values)) => {
                    result.extend(values.iter().filter_map(Value::as_str).map(str::to_owned))
                }
                _ => {}
            }
        }
    }
    result
}

fn substitute(value: &str, replacements: &HashMap<&str, String>) -> String {
    let mut result = value.to_owned();
    for (placeholder, replacement) in replacements {
        result = result.replace(placeholder, replacement);
    }
    result
}

fn offline_uuid(username: &str) -> String {
    let digest = Md5::digest(format!("OfflinePlayer:{username}").as_bytes());
    let mut bytes: [u8; 16] = digest.into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

fn classpath_separator() -> &'static str {
    if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }
}

pub(super) fn java_executable() -> PathBuf {
    if let Some(path) = env::var_os("LEOCARD_JAVA_PATH") {
        return PathBuf::from(path);
    }
    if let Some(home) = env::var_os("JAVA_HOME") {
        let path = PathBuf::from(home)
            .join("bin")
            .join(if cfg!(target_os = "windows") {
                "javaw.exe"
            } else {
                "java"
            });
        if path.is_file() {
            return path;
        }
    }
    PathBuf::from(if cfg!(target_os = "windows") {
        "javaw.exe"
    } else {
        "java"
    })
}
