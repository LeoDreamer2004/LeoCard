use super::super::MusicState;
use super::{account::render_account, library::render_library, player::render_player};
use crate::app::presentation::{DANGER, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_ui::UiComposer;

pub(super) fn render_music(
    commands: &mut UiComposer,
    parent: Entity,
    state: &MusicState,
    assets: &UiAssets,
) {
    render_account(commands, parent, state, assets);
    if let Some(error) = &state.error {
        add_text(commands, parent, error, 14.0, DANGER, assets);
    }
    render_player(commands, parent, state, assets);
    if state.account.is_some() {
        render_library(commands, parent, state, assets);
    }
}
