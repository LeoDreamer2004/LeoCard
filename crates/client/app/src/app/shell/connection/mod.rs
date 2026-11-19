//! 建房、加入房间与连接页面。

mod actions;
mod files;
mod form;
mod lan;
mod plugin;
mod rooms;
mod state;
mod view;

pub(crate) use actions::*;
pub(crate) use files::*;
pub(crate) use form::InputField;
pub(crate) use lan::LanDiscovery;
pub(super) use plugin::*;
pub(crate) use rooms::{JoinRoomAssets, JoinRoomUiState, RoomDirectory};
pub(crate) use state::{HomeCategory, HomeUiState, can_edit_identity};
pub(crate) use view::*;

mod end_feedback;
pub(crate) use end_feedback::ConnectionEndFeedback;
