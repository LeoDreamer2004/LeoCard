mod background;
mod click_feedback;
mod confirmation;
mod cozy;
mod header;
mod modal;
mod page_heading;
mod plugin;
mod transition;

use background::AmbientBackgroundPlugin;
pub(crate) use background::add_page_background;
use click_feedback::ClickFeedbackPlugin;
pub(crate) use confirmation::*;
pub(crate) use cozy::*;
pub(super) use header::Header;
pub(crate) use modal::*;
pub(crate) use page_heading::add_page_back_title;
pub(crate) use plugin::ShellUiPlugin;
pub(crate) use transition::*;

mod network;
pub(crate) use network::NetworkUiContext;
