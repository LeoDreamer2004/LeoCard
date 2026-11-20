#[cfg(feature = "desktop")]
mod detection;
#[cfg(feature = "desktop")]
mod monitor;
mod notifications;
mod plugin;
mod view;

pub(super) use plugin::McProcessPresencePlugin;
pub(super) use view::add_process_presence_overlay;
