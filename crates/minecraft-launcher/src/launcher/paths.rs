use std::env;
use std::path::{Component, Path, PathBuf};

pub(super) fn default_root() -> Result<PathBuf, String> {
    if let Some(root) = env::var_os("LEOCARD_MINECRAFT_DIR") {
        return Ok(PathBuf::from(root));
    }
    #[cfg(target_os = "windows")]
    let base = env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")));
    base.map(|path| path.join("leocard").join("minecraft"))
        .ok_or_else(|| "无法确定 Minecraft 数据目录".to_owned())
}

pub(super) fn safe_segment(value: &str) -> Result<&str, String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'+'))
        || value == "."
        || value == ".."
    {
        return Err(format!("无效的 Minecraft 文件名：{value}"));
    }
    Ok(value)
}

pub(super) fn safe_relative(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("下载路径不安全：{value}"));
    }
    Ok(path.to_path_buf())
}

pub(super) fn maven_path(name: &str) -> Result<PathBuf, String> {
    let mut parts = name.split(':');
    let group = parts.next().ok_or("库名称缺少 group")?;
    let artifact = safe_segment(parts.next().ok_or("库名称缺少 artifact")?)?;
    let version = safe_segment(parts.next().ok_or("库名称缺少 version")?)?;
    let classifier = parts.next().map(safe_segment).transpose()?;
    if parts.next().is_some() {
        return Err(format!("不支持的库名称：{name}"));
    }
    let mut path = PathBuf::new();
    for segment in group.split('.') {
        path.push(safe_segment(segment)?);
    }
    path.push(artifact);
    path.push(version);
    let file = match classifier {
        Some(classifier) => format!("{artifact}-{version}-{classifier}.jar"),
        None => format!("{artifact}-{version}.jar"),
    };
    path.push(file);
    Ok(path)
}
