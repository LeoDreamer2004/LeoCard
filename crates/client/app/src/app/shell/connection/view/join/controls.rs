use crate::app::presentation::game_side_control_scale;
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    ConnectionUiAction, CozyButtonVariant, UiAction, add_cozy_button_variant_scaled,
    add_cozy_disabled_button_scaled,
};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn row(commands: &mut UiComposer, parent: Entity) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            column_gap: px(8),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    )
}

pub(super) fn footer(commands: &mut UiComposer, parent: Entity) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: px(840),
            max_width: percent(94),
            align_self: AlignSelf::Center,
            flex_shrink: 0.0,
            padding: UiRect::axes(px(0), px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        None,
    )
}

pub(super) fn button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: Option<ConnectionUiAction>,
    assets: &UiAssets,
) {
    colored_button(
        commands,
        parent,
        label,
        action,
        CozyButtonVariant::Neutral,
        assets,
    );
}

pub(super) fn colored_button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: Option<ConnectionUiAction>,
    variant: CozyButtonVariant,
    assets: &UiAssets,
) {
    let scale = game_side_control_scale();
    // Flex owns the slot width. The shared sliced skin stretches inside it without
    // letting the source image's aspect ratio change the button or its text size.
    let slot = spawn_node(
        commands,
        parent,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            ..default()
        },
        None,
    );
    if let Some(action) = action {
        add_cozy_button_variant_scaled(
            commands,
            slot,
            label,
            UiAction::Connection(action),
            assets,
            percent(100),
            42.0 * scale,
            variant,
            scale,
        );
    } else {
        add_cozy_disabled_button_scaled(
            commands,
            slot,
            label,
            assets,
            percent(100),
            42.0 * scale,
            scale,
        );
    }
}
