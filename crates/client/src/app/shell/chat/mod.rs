//! 聊天界面、消息同步与气泡演出。

pub(crate) mod actions;
mod presentation;
mod state;
mod view;

pub(crate) use presentation::*;
pub(crate) use state::*;
pub(crate) use view::*;
