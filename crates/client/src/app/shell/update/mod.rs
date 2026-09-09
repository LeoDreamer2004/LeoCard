//! GitHub Release 自动更新、下载进度和更新窗口。

mod download;
mod state;
#[cfg(test)]
mod tests;
mod view;

use download::*;
pub(crate) use state::*;
pub(crate) use view::*;
