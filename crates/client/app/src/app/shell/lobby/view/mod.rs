//! 各游戏共享的大厅布局与席位视图。

mod header;
mod metrics;
mod page;
mod seats;
mod table;

pub(crate) use header::*;
use metrics::*;
pub(crate) use page::*;
use seats::*;
use table::*;
