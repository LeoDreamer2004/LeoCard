use super::{LanDiscovery, LanUiState};
use crate::app::games::SUPPORTED_GAMES;
use crate::app::presentation::{
    MUTED, PAGE_FRAME_TITLE, TEXT, add_avatar, add_text, add_title, page_frame_skin,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{
    ConnectionUiAction, UiAction, add_cozy_button, add_cozy_close_button, add_cozy_refresh_button,
};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_client::LanRoom;
use leocard_ui::{PageFrame, UiComposer, UiSceneKey, spawn_node};

#[derive(Component)]
pub(crate) struct LanRoomList;

pub(in crate::app::shell::connection) fn render_lan_rooms(
    commands: &mut UiComposer,
    parent: Entity,
    discovery: &LanDiscovery,
    state: &LanUiState,
    assets: &UiAssets,
) -> Entity {
    let mut frame = PageFrame::new(page_frame_skin(assets));
    frame.node.width = px(640);
    frame.node.max_width = percent(92);
    frame.node.height = px(480);
    frame.node.max_height = percent(85);
    frame.heading.height = px(48);
    frame.heading.column_gap = px(10);
    let frame = frame.mount(commands, parent);
    let heading = frame.heading;
    let title = add_title(
        commands,
        heading,
        "局域网联机",
        24.0,
        PAGE_FRAME_TITLE,
        assets,
    );
    commands.entity(title).insert(Node {
        flex_grow: 1.0,
        ..default()
    });
    add_cozy_refresh_button(
        commands,
        heading,
        UiAction::Connection(ConnectionUiAction::RefreshLan),
        assets,
    );
    add_cozy_close_button(
        commands,
        heading,
        UiAction::Connection(ConnectionUiAction::CloseLan),
        assets,
    );
    let list = commands.child(
        frame.content,
        (
            LanRoomList,
            ScrollPosition(Vec2::new(0.0, state.scroll_y)),
            RelativeCursorPosition::default(),
            Node {
                width: percent(100),
                min_height: px(0),
                flex_grow: 1.0,
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                ..default()
            },
        ),
    );
    if discovery.rooms().is_empty() {
        let empty = spawn_node(
            commands,
            list,
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            None,
        );
        add_text(
            commands,
            empty,
            discovery.error().unwrap_or("正在搜索局域网房间…"),
            16.0,
            MUTED,
            assets,
        );
    } else {
        for room in discovery.rooms() {
            render_room(commands, list, room, discovery, assets);
        }
    }
    frame.panel
}

fn render_room(
    commands: &mut UiComposer,
    parent: Entity,
    room: &LanRoom,
    discovery: &LanDiscovery,
    assets: &UiAssets,
) {
    let row = commands.child(
        parent,
        (
            UiSceneKey(room.server_id.to_string()),
            Node {
                width: percent(100),
                min_height: px(88),
                flex_shrink: 0.0,
                padding: UiRect::all(px(12)),
                align_items: AlignItems::Center,
                column_gap: px(14),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.06, 0.08, 0.4)),
        ),
    );
    add_avatar(
        commands,
        row,
        &room.info.host_name,
        discovery.avatar(room.server_id),
        54.0,
        assets,
    );
    let labels = spawn_node(
        commands,
        row,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    add_text(commands, labels, &room.info.host_name, 18.0, TEXT, assets);
    let game = SUPPORTED_GAMES
        .iter()
        .find(|game| game.kind == room.info.game)
        .expect("registered game");
    let status = if room.info.playing {
        "游戏中"
    } else {
        "等待中"
    };
    add_text(
        commands,
        labels,
        format!("{}  {status}", game.title),
        13.0,
        MUTED,
        assets,
    );
    add_cozy_button(
        commands,
        row,
        "加入",
        UiAction::Connection(ConnectionUiAction::JoinDiscovered(room.server_id)),
        assets,
        px(86),
        44.0,
    );
}
