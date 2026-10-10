//! 错误提示与断线覆盖层的视图构建。

use super::{
    PlayErrorPopup, PlayErrorPopupImage, PlayErrorPopupText, PlayErrorToast,
    play_error_toast_visual,
};
use crate::app::presentation::{ACCENT, MUTED, PanelSkin, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::add_cozy_panel_with_skin;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::VisualBox;

pub(crate) const WARNING_TOAST_TEXT: Color = Color::srgb(0.97, 0.96, 1.0);

pub(crate) fn add_play_error_popup(
    commands: &mut Commands,
    parent: Entity,
    message: &str,
    toast: &PlayErrorToast,
    assets: &UiAssets,
) {
    let visual = play_error_toast_visual(toast);
    let anchor = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(anchor);
    let popup = commands
        .spawn(Node {
            width: Val::Auto,
            min_width: px(360),
            max_width: percent(82),
            min_height: px(72),
            padding: UiRect {
                left: px(60),
                right: px(24),
                top: px(4),
                bottom: px(18),
            },
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(anchor).add_child(popup);
    commands.entity(popup).insert((
        PlayErrorPopup,
        UiTransform::from_translation(Val2::px(visual.x, visual.y)),
        GlobalZIndex(1500),
        Pickable::IGNORE,
    ));
    let mut image = ImageNode::new(assets.home.warning_toast.clone()).with_mode(
        NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect {
                min_inset: Vec2::new(120.0, 100.0),
                max_inset: Vec2::new(28.0, 16.0),
            },
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.48,
        }),
    );
    image.rect = Some(Rect::from_corners(
        Vec2::new(4.5, 260.5),
        Vec2::new(635.5, 379.5),
    ));
    image.visual_box = VisualBox::BorderBox;
    image.color = Color::WHITE.with_alpha(visual.opacity);
    let background = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            image,
            PlayErrorPopupImage,
            ZIndex(-1),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(popup).add_child(background);
    let text = add_text(
        commands,
        popup,
        message,
        16.0,
        WARNING_TOAST_TEXT.with_alpha(visual.opacity),
        assets,
    );
    commands.entity(text).insert(PlayErrorPopupText);
}

pub(crate) fn add_reconnecting_overlay(
    commands: &mut Commands,
    parent: Entity,
    status: &str,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.48)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(1400), Pickable::default()));
    let panel = add_cozy_panel_with_skin(
        commands,
        overlay,
        Node {
            width: px(520),
            max_width: percent(82),
            min_height: px(128),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
            ..default()
        },
        PanelSkin::Popup,
        assets,
    );
    add_text(commands, panel, "正在重新连接房主", 24.0, ACCENT, assets);
    add_text(commands, panel, status, 15.0, TEXT, assets);
    add_text(
        commands,
        panel,
        "连接恢复后会自动同步牌局，无需重新操作。",
        13.0,
        MUTED,
        assets,
    );
}
