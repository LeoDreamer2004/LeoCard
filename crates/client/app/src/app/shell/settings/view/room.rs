use super::section::add_settings_separator;
use super::toggle::add_settings_toggle_scaled;
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, MUTED, TEXT, add_text, add_title};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    CozyButtonVariant, SettingsUiAction, UiAction, add_cozy_button_variant_scaled,
};
use bevy::prelude::*;
use leocard_client::ClientModel;
use leocard_ui::{UiComposer, spawn_node};

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
    let scale = GAME_SIDE_CONTROL_SCALE;
    let is_host = model.is_host();
    add_title(commands, parent, "房间管理", 20.0 * scale, TEXT, assets);
    if is_host {
        add_settings_toggle_scaled(
            commands,
            parent,
            "禁止旁观玩家互动",
            room.spectator_interactions_blocked,
            SettingsUiAction::ToggleSpectatorInteractions,
            assets,
            scale,
        );
    } else {
        add_text(
            commands,
            parent,
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
    add_text(
        commands,
        parent,
        "限制旁观玩家送花、扔蛋等道具互动",
        12.0 * scale,
        MUTED,
        assets,
    );
    add_settings_separator(commands, parent);
    if is_host {
        add_text(
            commands,
            parent,
            "踢出参赛玩家后由系统接管其剩余操作",
            12.0 * scale,
            MUTED,
            assets,
        );
    }
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
