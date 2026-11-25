mod about;
#[cfg(not(target_arch = "wasm32"))]
mod ai;
mod appearance;
mod controls;
mod modal;
mod notifications;
mod room;
mod section;
mod slider;
mod sound;
mod toggle;

pub(crate) use modal::SettingsModal;
