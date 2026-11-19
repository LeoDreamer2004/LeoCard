use super::super::input::ConnectionInput;
use super::controls::{button, colored_button, row};
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::connection::rooms::{JoinRoomDialog, JoinRoomDialogKind};
use crate::app::shell::{ConnectionUiAction, CozyButtonVariant, InputField};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    dialog: &JoinRoomDialog,
    connecting: bool,
    assets: &UiAssets,
) {
    let form = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        None,
    );
    match &dialog.kind {
        JoinRoomDialogKind::Delete(_) => {
            add_text(
                commands,
                form,
                format!("确定从列表中删除“{}”？", dialog.name),
                16.0,
                TEXT,
                assets,
            );
        }
        kind => {
            if !matches!(kind, JoinRoomDialogKind::Direct) {
                add_text(commands, form, "房间名称", 16.0, TEXT, assets);
                ConnectionInput::new(&dialog.name, InputField::RoomName, assets)
                    .render(commands, form);
            }
            add_text(commands, form, "房间地址", 16.0, TEXT, assets);
            ConnectionInput::new(&dialog.address, InputField::JoinAddress, assets)
                .render(commands, form);
            add_text(
                commands,
                form,
                "IP 或域名加端口，例如 192.168.1.20:52300",
                13.0,
                MUTED,
                assets,
            );
        }
    }
    let (label, action, variant) = match dialog.kind {
        JoinRoomDialogKind::Direct => (
            if connecting {
                "连接中…"
            } else {
                "加入房间"
            },
            ConnectionUiAction::JoinRoom,
            CozyButtonVariant::Primary,
        ),
        JoinRoomDialogKind::Delete(_) => (
            "删除",
            ConnectionUiAction::ConfirmDeleteRoom,
            CozyButtonVariant::Danger,
        ),
        _ => (
            "保存",
            ConnectionUiAction::SaveRoom,
            CozyButtonVariant::Primary,
        ),
    };
    let actions = row(commands, parent);
    colored_button(
        commands,
        actions,
        label,
        (!connecting).then_some(action),
        variant,
        assets,
    );
    button(
        commands,
        actions,
        "取消",
        Some(ConnectionUiAction::CancelRoomEdit),
        assets,
    );
}
