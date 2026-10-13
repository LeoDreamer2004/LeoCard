use super::java::JavaLaunch;
use std::path::Path;
use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

pub(in super::super) fn minecraft_running() -> Option<bool> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .without_tasks()
            .with_cmd(UpdateKind::Always)
            .with_exe(UpdateKind::Always)
            .with_cwd(UpdateKind::Always),
    );
    if system.processes().is_empty() {
        return None;
    }
    let mut unreadable = false;
    for process in system.processes().values() {
        match minecraft_process(process) {
            Some(true) => return Some(true),
            None => unreadable = true,
            Some(false) => {}
        }
    }
    (!unreadable).then_some(false)
}

fn minecraft_process(process: &Process) -> Option<bool> {
    let name = process
        .exe()
        .and_then(Path::file_name)
        .unwrap_or_else(|| process.name())
        .to_string_lossy()
        .to_ascii_lowercase();
    if matches!(
        name.as_str(),
        "minecraft.windows.exe" | "minecraft.windows" | "minecraft.exe"
    ) {
        return Some(true);
    }
    if !matches!(name.as_str(), "java" | "javaw" | "java.exe" | "javaw.exe") {
        return Some(false);
    }
    if process.cmd().is_empty() {
        // A permissions failure is not evidence that Minecraft was closed.
        return None;
    }
    JavaLaunch::from_process(process.cmd(), process.cwd()).minecraft_client()
}
