use super::controls::{button, footer, row};
use super::entry::{RoomRow, entry, label};
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{DANGER, MUTED, READY};
use crate::app::runtime::UiAssets;
use crate::app::shell::ConnectionUiAction;
use crate::app::shell::connection::lan::LanDiscovery;
use crate::app::shell::connection::rooms::{
    JoinRoomAssets, JoinRoomList, JoinRoomUiState, LanSearchPulse, RoomDirectory, RoomSelection,
    RoomSignal, RoomStatus,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_protocol::GameKind;
use leocard_ui::{Scrollbar, ScrollbarSkin, UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    parent: Entity,
    state: &JoinRoomUiState,
    rooms: &RoomDirectory,
    discovery: &LanDiscovery,
    assets: &UiAssets,
    skin: &JoinRoomAssets,
) {
    let band = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_basis: px(0),
            min_height: px(0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    let body = spawn_node(
        commands,
        band,
        Node {
            width: px(900),
            max_width: percent(94),
            height: percent(100),
            min_height: px(0),
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let list = commands.keyed_child(
        body,
        "rooms",
        (
            JoinRoomList,
            ScrollPosition(Vec2::new(0.0, state.scroll_y)),
            RelativeCursorPosition::default(),
            Node {
                flex_grow: 1.0,
                flex_basis: px(0),
                min_width: px(0),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                row_gap: px(8),
                ..default()
            },
        ),
    );
    for room in rooms.entries() {
        let (status, color, info) = match rooms.status(&room.address) {
            Some(RoomStatus::Online { info, .. }) => (
                if info.playing {
                    "游戏中"
                } else {
                    "等待中"
                },
                READY,
                Some(info),
            ),
            Some(RoomStatus::Offline) => ("离线", DANGER, None),
            _ => ("正在查询…", MUTED, None),
        };
        let subtitle = if let Some(info) = info {
            format!("{}  {}", game_name(info.game), info.host_name)
        } else if let Some(host) = &room.host {
            format!("{}  {}", game_name(host.game), host.name)
        } else {
            "尚无房主信息".to_owned()
        };
        entry(
            commands,
            list,
            RoomRow {
                selection: RoomSelection::Saved(room.address.clone()),
                title: &room.name,
                subtitle: &subtitle,
                address: &room.address,
                status,
                color,
                avatar_name: room.host.as_ref().map_or(&room.name, |host| &host.name),
                avatar: rooms.avatar(room),
                signal: room_signal(rooms.status(&room.address)),
            },
            state,
            assets,
            skin,
        );
    }
    let separator = spawn_node(
        commands,
        list,
        Node {
            width: percent(100),
            min_height: px(62),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    label(
        commands,
        separator,
        "正在搜索局域网中的房间",
        15.0,
        MUTED,
        assets,
    );
    let dots = spawn_node(
        commands,
        separator,
        Node {
            width: percent(100),
            height: px(10),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    for index in 0..3 {
        commands.child(
            dots,
            (
                Node {
                    width: px(6),
                    height: px(6),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                BackgroundColor(MUTED.with_alpha(if index == 0 { 1.0 } else { 0.25 })),
                LanSearchPulse(index),
                Pickable::IGNORE,
            ),
        );
    }
    for room in discovery.rooms() {
        let address = room.address.to_string();
        if rooms.entries().iter().any(|saved| saved.address == address) {
            continue;
        }
        let title = format!("{}的房间", room.info.host_name);
        let subtitle = game_name(room.info.game);
        entry(
            commands,
            list,
            RoomRow {
                selection: RoomSelection::Lan(room.server_id),
                title: &title,
                subtitle,
                address: &address,
                status: if room.info.playing {
                    "游戏中"
                } else {
                    "等待中"
                },
                color: READY,
                avatar_name: &room.info.host_name,
                avatar: discovery.avatar(room.server_id),
                signal: room_signal(rooms.status(&address)),
            },
            state,
            assets,
            skin,
        );
    }
    if let Some(error) = discovery.error() {
        label(commands, list, error, 14.0, DANGER, assets);
    }
    Scrollbar::spawn(commands, body, list, ScrollbarSkin::Colored(MUTED));
    let selected = state.selected.as_ref().filter(|selection| match selection {
        RoomSelection::Saved(address) => {
            rooms.entries().iter().any(|room| &room.address == address)
        }
        RoomSelection::Lan(id) => discovery.rooms().iter().any(|room| room.server_id == *id),
    });
    let saved = !state.connecting && matches!(selected, Some(RoomSelection::Saved(_)));
    let footer = footer(commands, parent);
    let primary = row(commands, footer);
    button(
        commands,
        primary,
        if state.connecting {
            "连接中…"
        } else {
            "加入房间"
        },
        selected
            .filter(|_| !state.connecting)
            .map(|_| ConnectionUiAction::JoinSelected),
        assets,
    );
    button(
        commands,
        primary,
        "直接连接",
        (!state.connecting).then_some(ConnectionUiAction::DirectConnect),
        assets,
    );
    button(
        commands,
        primary,
        "添加房间",
        (!state.connecting).then_some(ConnectionUiAction::AddRoom),
        assets,
    );
    let secondary = row(commands, footer);
    for (label, action) in [
        ("编辑", saved.then_some(ConnectionUiAction::EditRoom)),
        ("删除", saved.then_some(ConnectionUiAction::DeleteRoom)),
        (
            "刷新",
            (!state.connecting).then_some(ConnectionUiAction::RefreshRooms),
        ),
        ("取消", Some(ConnectionUiAction::ToggleJoin)),
    ] {
        button(commands, secondary, label, action, assets);
    }
}

fn game_name(kind: GameKind) -> &'static str {
    SUPPORTED_GAMES
        .iter()
        .find(|game| game.kind == kind)
        .map_or("房间", |game| game.title)
}

fn room_signal(status: Option<&RoomStatus>) -> RoomSignal {
    match status {
        Some(RoomStatus::Online { latency_ms, .. }) => RoomSignal::Online(*latency_ms),
        Some(RoomStatus::Offline) => RoomSignal::Offline,
        _ => RoomSignal::Checking,
    }
}
