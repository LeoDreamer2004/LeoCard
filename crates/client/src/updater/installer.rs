//! 在独立辅助进程中替换正在运行的客户端二进制。

use super::receipt::UpdateReceipt;
use leocard_protocol::ProfileId;
use std::{
    env::{self, consts},
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
    thread,
    time::Duration,
};

const APPLY_UPDATE_ARGUMENT: &str = "--leocard-apply-update";
const REPLACE_ATTEMPTS: usize = 120;
const REPLACE_RETRY_DELAY: Duration = Duration::from_millis(250);

/// 若当前进程由游戏作为更新辅助进程启动，则完成替换并阻止 Bevy 启动。
pub(crate) fn run_if_requested() -> bool {
    let mut arguments = env::args_os().skip(1);
    if arguments.next().as_deref() != Some(APPLY_UPDATE_ARGUMENT.as_ref()) {
        return false;
    }

    let staged = arguments.next().map(PathBuf::from);
    let target = arguments.next().map(PathBuf::from);
    let receipt = arguments.next().map(PathBuf::from);
    let result = match (staged.as_deref(), target.as_deref(), receipt.as_deref()) {
        (Some(staged), Some(target), Some(receipt)) => apply_update(staged, target, receipt),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "更新辅助进程缺少文件路径",
        )),
    };
    if let Err(error) = result {
        write_update_error(target.as_deref(), &error);
    }
    cleanup_helper_after_exit();
    true
}

/// 复制一个不受当前可执行文件锁影响的辅助进程，然后由调用方退出游戏。
pub(crate) fn launch_installer(
    staged: &Path,
    version: &str,
    profile_id: ProfileId,
) -> Result<(), String> {
    let target = env::current_exe().map_err(|error| format!("无法定位当前程序：{error}"))?;
    if !staged.is_file() {
        return Err("已下载的更新文件不存在，请重新下载".to_owned());
    }
    let helper_name = format!("leocard-updater-{}{}", process::id(), consts::EXE_SUFFIX);
    let helper = env::temp_dir().join(helper_name);
    let receipt = UpdateReceipt::stage(staged, version, profile_id)?;
    let launched = fs::copy(&target, &helper).and_then(|_| {
        Command::new(&helper)
            .arg(APPLY_UPDATE_ARGUMENT)
            .arg(staged)
            .arg(&target)
            .arg(&receipt)
            .spawn()
    });
    if let Err(error) = launched {
        let _ = fs::remove_file(&helper);
        let _ = fs::remove_file(&receipt);
        return Err(format!("无法启动更新辅助程序：{error}"));
    }
    Ok(())
}

fn apply_update(staged: &Path, target: &Path, receipt: &Path) -> io::Result<()> {
    // 给主进程时间关闭窗口并释放 Windows 对 exe 的独占锁。
    thread::sleep(Duration::from_millis(600));
    let backup = backup_path(target);
    if backup.exists() {
        fs::remove_file(&backup)?;
    }

    let mut last_error = None;
    for _ in 0..REPLACE_ATTEMPTS {
        match fs::rename(target, &backup) {
            Ok(()) => {
                last_error = None;
                break;
            }
            Err(error) => {
                last_error = Some(error);
                thread::sleep(REPLACE_RETRY_DELAY);
            }
        }
    }
    if let Some(error) = last_error {
        return Err(error);
    }

    if let Err(error) = fs::rename(staged, target) {
        let _ = fs::rename(&backup, target);
        return Err(error);
    }

    if let Err(error) = UpdateReceipt::complete(receipt, target) {
        write_update_error(Some(target), &error);
    }

    let working_directory = target.parent().unwrap_or_else(|| Path::new("."));
    // 新版本已经安装；若启动失败，保留它供用户稍后手动启动并报告错误。
    Command::new(target)
        .current_dir(working_directory)
        .spawn()?;
    let _ = fs::remove_file(backup);
    Ok(())
}

pub(super) fn backup_path(target: &Path) -> PathBuf {
    let mut name = target
        .file_name()
        .map_or_else(|| OsString::from("leocard"), OsString::from);
    name.push(".old");
    target.with_file_name(name)
}

fn write_update_error(target: Option<&Path>, error: &io::Error) {
    let path = target
        .and_then(Path::parent)
        .map(|parent| parent.join("leocard-update-error.txt"))
        .unwrap_or_else(|| env::temp_dir().join("leocard-update-error.txt"));
    let _ = fs::write(path, format!("LeoCard 更新错误：{error}\n"));
}

fn cleanup_helper_after_exit() {
    let Ok(helper) = env::current_exe() else {
        return;
    };
    if !is_helper_path(&helper) {
        return;
    }

    #[cfg(unix)]
    let _ = fs::remove_file(helper);
    #[cfg(target_os = "windows")]
    let _ = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "Start-Sleep -Milliseconds 800; Remove-Item -LiteralPath $env:LEOCARD_UPDATE_HELPER -Force -ErrorAction SilentlyContinue",
        ])
        .env("LEOCARD_UPDATE_HELPER", helper)
        .spawn();
}

pub(super) fn is_helper_path(path: &Path) -> bool {
    path.file_stem()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("leocard-updater-"))
}
