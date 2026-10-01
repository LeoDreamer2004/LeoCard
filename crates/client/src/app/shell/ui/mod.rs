mod background;
mod cozy;
mod header;
mod modal;
mod plugin;
mod transition;

use background::AmbientBackgroundPlugin;
pub(crate) use background::add_page_background;
pub(crate) use cozy::*;
pub(super) use header::Header;
pub(crate) use modal::*;
pub(crate) use plugin::ShellUiPlugin;
pub(crate) use transition::*;

mod network;
pub(crate) use network::NetworkUiContext;
