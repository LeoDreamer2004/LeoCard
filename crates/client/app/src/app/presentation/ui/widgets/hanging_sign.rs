//! 共用的悬挂木牌，内容与悬挂部分分开布局，保留手机上的文字缩放。

use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, decorate_panel, spawn_node};

pub(crate) struct HangingSignEntities {
    pub panel: Entity,
    pub content: Entity,
}

pub(crate) fn add_hanging_table_sign(
    commands: &mut UiComposer,
    parent: Entity,
    assets: &UiAssets,
    scale: f32,
    compact: bool,
    content_width: f32,
) -> HangingSignEntities {
    let height = if compact { 92.0 } else { 110.0 } + 32.0 * (scale - 1.0);
    let panel = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px((content_width + 40.0) * scale),
            height: px(height),
            padding: UiRect::top(px(height * 0.14)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    commands.entity(panel).insert(GlobalZIndex(30));
    // Keep artwork out of intrinsic image layout and out of the padded content box.
    // The skin fills the panel's outer bounds, including the ropes at y=0.
    decorate_panel(commands, panel, assets.tabletop.hanging_sign_image(scale));
    let content = spawn_node(
        commands,
        panel,
        Node {
            width: px(content_width),
            height: px(66),
            flex_shrink: 0.0,
            padding: UiRect::axes(px(10), px(5)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    commands.entity(content).insert((
        UiTransform::from_scale(Vec2::splat(scale)),
        Pickable::IGNORE,
    ));
    HangingSignEntities { panel, content }
}
