//! 错误提示与断线覆盖层的视图构建。

use super::*;

pub fn add_play_error_popup(
    commands: &mut Commands,
    parent: Entity,
    message: &str,
    toast: &PlayErrorToast,
    assets: &UiAssets,
) {
    let visual = play_error_toast_visual(toast);
    let popup = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: percent(28),
            right: percent(28),
            top: percent(50),
            min_height: px(62),
            padding: UiRect::axes(px(18), px(12)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(9)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.97 * visual.opacity)),
    );
    commands.entity(popup).insert((
        PlayErrorPopup,
        BorderColor::all(DANGER.with_alpha(0.9 * visual.opacity)),
        BoxShadow::new(
            Color::BLACK.with_alpha(0.45 * visual.opacity),
            px(2),
            px(5),
            px(0),
            px(8),
        ),
        UiTransform::from_translation(Val2::px(visual.x, visual.y)),
        GlobalZIndex(1500),
        FocusPolicy::Pass,
    ));
    let text = add_text(
        commands,
        popup,
        message,
        18.0,
        DANGER.with_alpha(visual.opacity),
        assets,
    );
    commands.entity(text).insert(PlayErrorPopupText);
}

pub fn add_reconnecting_overlay(
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
        .insert((GlobalZIndex(1400), FocusPolicy::Block));
    let panel = add_panel(
        commands,
        overlay,
        Node {
            width: px(520),
            max_width: percent(82),
            min_height: px(128),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
            ..default()
        },
        HEADER_BG,
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
