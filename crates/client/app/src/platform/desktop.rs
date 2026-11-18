use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

pub fn pick_image(title: &str) -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title(title)
        .add_filter("图片", &["png", "jpg", "jpeg"])
        .pick_file())
}

pub fn open_url(url: &str) -> Result<(), String> {
    launch(url.as_ref())
}

pub fn open_directory(path: &Path) -> Result<(), String> {
    launch(path.as_os_str())
}

fn launch(target: &OsStr) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe").arg(target).spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(target).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(target).spawn();
    #[cfg(not(any(target_os = "windows", unix)))]
    return Err("当前系统不支持自动打开网页".to_owned());

    let mut child = result.map_err(|error| format!("无法打开系统应用：{error}"))?;
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
