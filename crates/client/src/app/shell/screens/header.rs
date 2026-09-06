//! 顶栏及其连接状态、设置和个人资料入口。

use super::*;

pub fn add_header(
    commands: &mut Commands,
    root: Entity,
    client: Option<&ClientResource>,
    form: &ConnectionForm,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let header = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            height: px(64),
            padding: UiRect::axes(px(24), px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        Some(HEADER_BG),
    );
    let left = spawn_node(
        commands,
        header,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    add_text(commands, left, "LeoCard", 25.0, ACCENT, assets);
    if let Some(host_port) = client.and_then(|client| client.0.model().host_port()) {
        let divider = spawn_node(
            commands,
            left,
            Node {
                width: px(1),
                height: px(24),
                ..default()
            },
            Some(BORDER),
        );
        commands.entity(divider).insert(FocusPolicy::Pass);
        add_text(
            commands,
            left,
            format!("端口 {host_port}"),
            15.0,
            MUTED,
            assets,
        );
    }
    let right = spawn_node(
        commands,
        header,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    add_header_button(
        commands,
        right,
        "游戏设置",
        UiAction::ToggleSettings,
        assets,
    );
    if client.is_some_and(|client| client.0.model().game_snapshot().is_some()) {
        add_header_exit_button(commands, right, "退出游戏", assets);
    }
    add_profile_avatar_button(commands, right, form, assets, avatars);
}
