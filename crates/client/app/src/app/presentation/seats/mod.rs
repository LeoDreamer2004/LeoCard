//! 各游戏共享的座位方向、人数分布与放置方式。

mod arc;
mod lobby;
mod rows;
mod table;
mod trick;

pub(crate) use arc::*;
pub(crate) use lobby::*;
pub(crate) use rows::*;
pub(crate) use table::*;
pub(crate) use trick::*;
