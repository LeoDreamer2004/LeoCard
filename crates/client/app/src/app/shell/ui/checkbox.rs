#[cfg(not(target_os = "android"))]
use crate::app::presentation::{TEXT, add_text};
use crate::app::runtime::UiAssets;
#[cfg(not(target_os = "android"))]
use crate::app::shell::UiAction;
#[cfg(not(target_os = "android"))]
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
#[cfg(not(target_os = "android"))]
use bevy::ui_widgets::Button;
#[cfg(not(target_os = "android"))]
use leocard_ui::UiComposer;

#[derive(Component)]
pub(crate) struct CozyCheckbox {
    pub icon: Entity,
    pub selected: bool,
}

#[cfg(not(target_os = "android"))]
pub(crate) fn add_cozy_checkbox(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    selected: bool,
    action: UiAction,
    assets: &UiAssets,
) -> Entity {
    let button = commands.child(
        parent,
        (
            Button,
            action,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(6),
                flex_shrink: 0.0,
                ..default()
            },
        ),
    );
    let icon = commands.child(
        button,
        (
            ImageNode::new(if selected {
                assets.home.checkbox_selected.clone()
            } else {
                assets.home.checkbox.clone()
            }),
            Node {
                width: px(28),
                height: px(28),
                flex_shrink: 0.0,
                ..default()
            },
            Pickable::IGNORE,
        ),
    );
    commands
        .entity(button)
        .insert(CozyCheckbox { icon, selected });
    let text = add_text(commands, button, label, 15.0, TEXT, assets);
    commands.entity(text).insert(Pickable::IGNORE);
    button
}

pub(crate) fn update_cozy_checkbox_hover(
    assets: Res<UiAssets>,
    toggles: Query<(&PickingInteraction, &CozyCheckbox)>,
    mut images: Query<&mut ImageNode>,
) {
    for (interaction, toggle) in &toggles {
        let texture = match (toggle.selected, *interaction != PickingInteraction::None) {
            (false, false) => &assets.home.checkbox,
            (false, true) => &assets.home.checkbox_highlighted,
            (true, false) => &assets.home.checkbox_selected,
            (true, true) => &assets.home.checkbox_selected_highlighted,
        };
        if let Ok(mut image) = images.get_mut(toggle.icon)
            && image.image != *texture
        {
            image.image = texture.clone();
        }
    }
}
