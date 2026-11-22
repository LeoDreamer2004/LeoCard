use super::section::add_settings_separator;
use super::toggle::add_settings_toggle_scaled;
use crate::app::presentation::{
    MUTED, TEXT, add_text, add_title, game_side_control_scale, rule_help_style,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, SettingsUiAction, ShellOverlayLayer, UiAction,
    add_cozy_button_variant_scaled,
};
use bevy::prelude::*;
use leocard_client::ClientModel;
use leocard_ui::{UiComposer, add_help_tooltip, spawn_node};

pub(super) fn render_room_settings(
    commands: &mut UiComposer,
    parent: Entity,
    model: Option<&ClientModel>,
    assets: &UiAssets,
) {
    let Some(model) = model else {
        add_text(
            commands,
            parent,
            "加入房间后可查看房间管理",
            16.0,
            MUTED,
            assets,
        );
        return;
    };
    let Some(room) = model.room_moderation() else {
        return;
    };
    let scale = game_side_control_scale();
    let is_host = model.is_host();
    add_title(commands, parent, "房间管理", 20.0 * scale, TEXT, assets);
    let restriction = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(48.0 * scale),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(6.0 * scale),
            ..default()
        },
        None,
    );
    if is_host {
        let toggle = add_settings_toggle_scaled(
            commands,
            restriction,
            "禁止旁观玩家互动",
            room.spectator_interactions_blocked,
            SettingsUiAction::ToggleSpectatorInteractions,
            assets,
            scale,
        );
        commands
            .entity(toggle)
            .entry::<Node>()
            .and_modify(|mut node| node.flex_grow = 0.0);
    } else {
        add_text(
            commands,
            restriction,
            if room.spectator_interactions_blocked {
                "房主已禁止旁观玩家互动"
            } else {
                "旁观玩家可以互动"
            },
            16.0 * scale,
            MUTED,
            assets,
        );
    }
    let tooltip = add_help_tooltip(
        commands,
        restriction,
        "限制旁观玩家送花、扔蛋等道具互动。踢出参赛玩家后由系统接管其剩余操作。",
        rule_help_style(assets),
    );
    commands
        .entity(tooltip)
        .insert(GlobalZIndex(ShellOverlayLayer::Dialog.z_index().0 + 1));
    add_settings_separator(commands, parent);
    for participant in &room.participants {
        let row = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                min_height: px(48.0 * scale),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(8.0 * scale),
                ..default()
            },
            None,
        );
        let identity = spawn_node(
            commands,
            row,
            Node {
                flex_direction: FlexDirection::Column,
                min_width: px(0),
                row_gap: px(2),
                ..default()
            },
            None,
        );
        add_text(
            commands,
            identity,
            &participant.player.name,
            15.0 * scale,
            TEXT,
            assets,
        );
        let role = if Some(participant.player.id) == room.host {
            "房主"
        } else if participant.is_bot {
            "机器人"
        } else if participant.spectating {
            "旁观玩家"
        } else {
            "参赛玩家"
        };
        let status = if participant.connected {
            ""
        } else {
            " 已断线"
        };
        add_text(
            commands,
            identity,
            format!("{role}{status}"),
            11.0 * scale,
            MUTED,
            assets,
        );
        if is_host && Some(participant.player.id) != room.host && !participant.is_bot {
            add_cozy_button_variant_scaled(
                commands,
                row,
                "踢出",
                UiAction::Settings(SettingsUiAction::RequestKickPlayer(participant.player.id)),
                assets,
                px(64.0 * scale),
                38.0 * scale,
                CozyButtonVariant::Danger,
                scale,
            );
        }
    }
}
