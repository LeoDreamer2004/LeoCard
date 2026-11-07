use super::super::MusicUiAction;
use crate::app::runtime::UiAssets;
use crate::app::shell::{UiAction, add_cozy_button, add_cozy_disabled_button};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn row(commands: &mut UiComposer, parent: Entity) -> Entity {
    spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(8),
            row_gap: px(6),
            ..default()
        },
        None,
    )
}

pub(super) fn button(
    commands: &mut UiComposer,
    parent: Entity,
    label: &str,
    action: MusicUiAction,
    enabled: bool,
    assets: &UiAssets,
) {
    let height = if cfg!(target_os = "android") {
        44.0
    } else {
        36.0
    };
    let width = px((label.chars().count() as f32 * 15.0 + 28.0).max(62.0));
    if enabled {
        add_cozy_button(
            commands,
            parent,
            label,
            UiAction::Music(action),
            assets,
            width,
            height,
        );
    } else {
        add_cozy_disabled_button(commands, parent, label, assets, width, height);
    }
}
