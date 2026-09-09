//! 跨游戏共用的错误与断线覆盖层。

mod feedback;
mod rejection;
mod state;
mod view;

pub(crate) use feedback::*;
pub(crate) use rejection::*;
pub(crate) use state::*;
pub(crate) use view::*;
