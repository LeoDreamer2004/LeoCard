//! GitHub Release 自动更新、下载进度和更新窗口。

mod download;
mod plugin;
mod state;
#[cfg(test)]
mod tests;
mod view;

use download::*;
pub(super) use plugin::*;
pub(crate) use state::*;
pub(crate) use view::*;
