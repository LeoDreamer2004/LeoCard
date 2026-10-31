mod background;
mod checkbox;
mod click_feedback;
mod configuration;
mod confirmation;
mod cozy;
mod dropdown;
mod header;
mod modal;
mod page_heading;
mod plugin;
#[cfg(not(target_os = "android"))]
mod switch;
mod transition;

use background::AmbientBackgroundPlugin;
pub(crate) use background::add_page_background;
pub(crate) use checkbox::*;
use click_feedback::ClickFeedbackPlugin;
pub(crate) use configuration::add_configuration_scroll;
pub(crate) use confirmation::*;
pub(crate) use cozy::*;
pub(crate) use dropdown::*;
use header::GameHeaderPlugin;
pub(super) use header::Header;
pub(crate) use modal::*;
pub(crate) use page_heading::add_page_back_title;
pub(crate) use plugin::ShellUiPlugin;
#[cfg(not(target_os = "android"))]
pub(crate) use switch::add_cozy_switch;
pub(crate) use transition::*;

mod network;
pub(crate) use network::NetworkUiContext;
