mod actions;
mod mode;
mod motion;
mod playback;
mod plugin;
mod state;
mod storage;
mod systems;
mod task;
mod view;

pub(crate) use actions::MusicUiAction;
pub(crate) use motion::MusicMotion;
pub(super) use plugin::MusicPlugin;
pub(crate) use state::MusicState;
pub(crate) use view::{render_mini_player, render_music_modal, render_music_settings};
