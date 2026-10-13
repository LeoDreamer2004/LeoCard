use super::{
    state::{TextInputEvent, TextInputSet},
    systems::{collect_submissions, focus_input_slot, sync_input_skin},
};
use bevy::{
    input_focus::tab_navigation::TabNavigationPlugin, prelude::*, text::EditableTextSystems,
    ui::UiSystems,
};

pub(crate) struct TextInputPlugin;

impl Plugin for TextInputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "android")]
        app.add_systems(PreUpdate, super::android::edit_native_text);
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
