//! 玩家互动投掷物与通用得分收集演出。

pub(crate) mod actions;
mod animation;
mod interaction;
mod menu;
mod score_capture;
mod state;

pub(crate) use animation::*;
pub(crate) use interaction::*;
pub(crate) use menu::*;
pub(crate) use score_capture::*;
pub(crate) use state::*;
