mod animation;
mod material;
mod plugin;
mod state;
#[cfg(test)]
mod tests;
mod view;

pub(super) use plugin::LobbyEntrancePlugin;
pub(crate) use state::LobbyEntranceState;
pub(crate) use view::LobbySeatedAvatar;
