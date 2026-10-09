use super::{
    retention::restore_editors,
    state::{RetainedEditors, TextInputEvent, TextInputSet},
    systems::{collect_submissions, focus_input_slot, sync_input_skin},
};
use crate::app::runtime::ClientUpdateSet;
use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, text::EditableTextSystems,
    ui::UiSystems,
};

pub(crate) struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TabNavigationPlugin)
            .add_observer(focus_input_slot)
            .add_observer(collect_submissions)
            .init_resource::<RetainedEditors>()
            .add_message::<TextInputEvent>()
            .configure_sets(
                PostUpdate,
                TextInputSet::Publish
                    .after(EditableTextSystems)
                    .before(UiSystems::PostLayout),
            )
            .add_systems(
                Update,
                restore_editors
                    .after(ClientUpdateSet::Rebuild)
                    .before(ClientUpdateSet::Animate),
            )
            .add_systems(
                PostUpdate,
                sync_input_skin
                    .after(TextInputSet::Publish)
                    .before(UiSystems::PostLayout),
            );
    }
}
