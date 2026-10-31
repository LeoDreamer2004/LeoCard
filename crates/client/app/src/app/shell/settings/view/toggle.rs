use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{CozyCheckbox, SettingsUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::UiComposer;

pub(super) fn add_settings_toggle(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    selected: bool,
    action: SettingsUiAction,
    assets: &UiAssets,
) {
    let button = commands.child(
        parent,
        (
            Button,
            UiAction::Settings(action),
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                height: px(48),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        ),
    );
    let icon = commands.child(
        button,
        (
            Node {
                width: px(34),
                height: px(34),
                ..default()
            },
            ImageNode::new(if selected {
                assets.home.checkbox_selected.clone()
            } else {
                assets.home.checkbox.clone()
            }),
            Pickable::IGNORE,
        ),
    );
    commands
        .entity(button)
        .insert(CozyCheckbox { icon, selected });
    add_text(commands, button, label, 16.0, TEXT, assets);
}
