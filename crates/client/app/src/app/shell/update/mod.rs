//! GitHub Release 自动更新、下载进度和更新窗口。

mod actions;
#[cfg(not(target_os = "android"))]
mod completion;

mod animation;

mod plugin;
mod state;
mod view;

use leocard_update::UpdateEvent;
pub(super) use plugin::*;
pub(crate) use state::*;
pub(crate) use view::*;

use animation::advance_update_modal;

pub(crate) use actions::UpdateUiAction;
