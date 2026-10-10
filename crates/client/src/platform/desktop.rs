use std::{path::PathBuf, process::Command, thread};

pub fn pick_image(title: &str) -> Result<Option<PathBuf>, String> {
    Ok(rfd::FileDialog::new()
        .set_title(title)
        .add_filter("图片", &["png", "jpg", "jpeg"])
        .pick_file())
}

pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let result = Command::new("explorer.exe").arg(url).spawn();
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(url).spawn();
    #[cfg(all(unix, not(target_os = "macos")))]
    let result = Command::new("xdg-open").arg(url).spawn();
    #[cfg(not(any(target_os = "windows", unix)))]
    return Err("当前系统不支持自动打开网页".to_owned());

    let mut child = result.map_err(|error| format!("无法打开系统浏览器：{error}"))?;
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
