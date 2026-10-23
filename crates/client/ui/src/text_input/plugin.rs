use super::{
    state::{TextInputEvent, TextInputSet},
    systems::{collect_submissions, focus_input_slot, sync_input_skin},
};
use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, text::EditableTextSystems,
    ui::UiSystems,
};

pub struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            super::password::sync_password_masks
                .after(TextInputSet::Publish)
                .after(UiSystems::Layout)
                .before(UiSystems::PostLayout),
        );
        app.add_plugins(TabNavigationPlugin)
            .add_observer(focus_input_slot)
            .add_observer(collect_submissions)
            .add_message::<TextInputEvent>()
            .configure_sets(
                PostUpdate,
                TextInputSet::Publish
                    .after(EditableTextSystems)
                    .before(UiSystems::PostLayout),
            )
            .add_systems(
                PostUpdate,
                sync_input_skin
                    .after(TextInputSet::Publish)
                    .before(UiSystems::PostLayout),
            );
    }
}
