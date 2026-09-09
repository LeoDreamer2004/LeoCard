//! 顶层连接、房间、设置与牌桌页面。

mod composition;
mod connection;
mod header;
mod lobby;
mod settings;
mod state;

pub(crate) use composition::*;
use connection::*;
use header::*;
pub(crate) use lobby::*;
use settings::*;
pub(crate) use state::*;
