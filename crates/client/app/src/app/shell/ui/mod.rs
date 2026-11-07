mod background;
mod checkbox;
mod click_feedback;
mod configuration;
mod confirmation;
mod cozy;
mod dropdown;
mod header;
mod liquid;
mod modal;
mod overlay_layer;
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
use liquid::LiquidUiPlugin;
pub(crate) use liquid::{LiquidKind, LiquidSurface};
pub(crate) use modal::*;
pub(crate) use overlay_layer::ShellOverlayLayer;
pub(crate) use page_heading::add_page_back_title;
pub(crate) use plugin::ShellUiPlugin;
#[cfg(not(target_os = "android"))]
pub(crate) use switch::add_cozy_switch;
pub(crate) use transition::*;

mod network;
pub(crate) use network::NetworkUiContext;
