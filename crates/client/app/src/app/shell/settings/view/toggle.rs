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
    add_settings_toggle_scaled(commands, parent, label, selected, action, assets, 1.0);
}

pub(super) fn add_settings_toggle_scaled(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    selected: bool,
    action: SettingsUiAction,
    assets: &UiAssets,
    scale: f32,
) {
    let button = commands.child(
        parent,
        (
            Button,
            UiAction::Settings(action),
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                height: px(48.0 * scale),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                column_gap: px(8.0 * scale),
                ..default()
            },
        ),
    );
    let icon = commands.child(
        button,
        (
            Node {
                width: px(34.0 * scale),
                height: px(34.0 * scale),
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
    add_text(commands, button, label, 16.0 * scale, TEXT, assets);
}
