#[cfg(not(target_os = "android"))]
mod detection;
#[cfg(not(target_os = "android"))]
mod monitor;
mod notifications;
mod plugin;
mod view;

pub(super) use plugin::MinecraftProcessPresencePlugin;
pub(super) use view::add_process_presence_overlay;
