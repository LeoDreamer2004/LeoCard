mod actions;
mod controls;
mod lobby;
mod playback;
mod players;
mod screen;
mod state;
pub(super) mod summary;
mod table;
#[cfg(test)]
mod tests;

pub(crate) use actions::MahjongLinkUiAction;
pub(crate) use screen::{MahjongLinkPlugin, MahjongLinkScreen};
