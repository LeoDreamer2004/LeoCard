use std::path::Path;
use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

pub(super) fn minecraft_running() -> Option<bool> {
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .without_tasks()
            .with_cmd(UpdateKind::Always)
            .with_exe(UpdateKind::Always),
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
    let arguments: Vec<_> = process
        .cmd()
        .iter()
        .map(|argument| argument.to_string_lossy().to_ascii_lowercase())
        .collect();
    let client_main = arguments.iter().any(|argument| {
        matches!(
            argument.as_str(),
            "net.minecraft.client.main.main"
                | "net.minecraft.client.minecraft"
                | "net.fabricmc.loader.impl.launch.knot.knotclient"
                | "net.fabricmc.loader.launch.knot.knotclient"
                | "org.quiltmc.loader.impl.launch.knot.knotclient"
        )
    });
    let forge_client = arguments.windows(2).any(|pair| {
        pair[0] == "--launchtarget"
            && matches!(
                pair[1].as_str(),
                "forgeclient" | "neoforgeclient" | "fmlclient"
            )
    });
    // Older launchwrapper and launcher-specific bootstraps retain these client arguments.
    let client_arguments = arguments.iter().any(|argument| argument == "--assetsdir")
        && arguments.iter().any(|argument| argument == "--username");
    Some(client_main || forge_client || client_arguments)
}
