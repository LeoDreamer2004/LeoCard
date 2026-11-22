use crate::app::presentation::{MUTED, TEXT, add_text};
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
    action: impl Into<Option<SettingsUiAction>>,
    assets: &UiAssets,
) -> Entity {
    add_settings_toggle_scaled(commands, parent, label, selected, action, assets, 1.0)
}

pub(super) fn add_settings_toggle_scaled(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    selected: bool,
    action: impl Into<Option<SettingsUiAction>>,
    assets: &UiAssets,
    scale: f32,
) -> Entity {
    let action = action.into();
    let enabled = action.is_some();
    let button = commands.child(
        parent,
        (Node {
            flex_grow: 1.0,
            min_width: px(0),
            height: px(48.0 * scale),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(8.0 * scale),
            ..default()
        },),
    );
    if let Some(action) = action {
        commands
            .entity(button)
            .insert((Button, UiAction::Settings(action)));
    }
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
            })
            .with_color(Color::WHITE.with_alpha(if enabled { 1.0 } else { 0.4 })),
            Pickable::IGNORE,
        ),
    );
    commands
        .entity(button)
        .insert(CozyCheckbox { icon, selected });
    add_text(
        commands,
        button,
        label,
        16.0 * scale,
        if enabled { TEXT } else { MUTED },
        assets,
    );
    button
}
