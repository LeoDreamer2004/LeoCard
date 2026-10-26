use super::input::sync_coin_input;
use bevy::prelude::*;
use leocard_ui::TextInputSet;

pub(crate) struct DeveloperCoinsPlugin;

impl Plugin for DeveloperCoinsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, sync_coin_input.in_set(TextInputSet::Publish));
    }
}
