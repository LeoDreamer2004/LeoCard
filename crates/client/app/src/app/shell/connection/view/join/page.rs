use super::super::page::ConnectionScreen;
use super::list;
use crate::app::shell::connection::lan::LanDiscovery;
use crate::app::shell::connection::rooms::{JoinRoomAssets, JoinRoomUiState, RoomDirectory};
use crate::app::shell::{ConnectionUiAction, PageTransitionElement, UiAction, add_page_back_title};
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

impl ConnectionScreen<'_> {
    pub(crate) fn render_join_page(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        state: &JoinRoomUiState,
        rooms: &RoomDirectory,
        discovery: &LanDiscovery,
        skin: &JoinRoomAssets,
    ) {
        let close = UiAction::Connection(ConnectionUiAction::ToggleJoin);
        let canvas = spawn_node(
            commands,
            root,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            },
            None,
        );
        let page = spawn_node(
            commands,
            canvas,
            Node {
                width: percent(100),
                max_width: px(1320),
                flex_grow: 1.0,
                min_height: px(0),
                padding: UiRect::all(px(if cfg!(target_os = "android") { 12 } else { 20 })),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        let heading = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                ..default()
            },
            None,
        );
        commands
            .entity(heading)
            .insert((PageTransitionElement::left(0), UiTransform::IDENTITY));
        add_page_back_title(commands, heading, "加入房间", close, self.assets);
        let body = spawn_node(
            commands,
            page,
            Node {
                width: percent(100),
                flex_grow: 1.0,
                flex_basis: px(0),
                min_height: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            None,
        );
        commands
            .entity(body)
            .insert((PageTransitionElement::right(1), UiTransform::IDENTITY));
        list::render(commands, body, state, rooms, discovery, self.assets, skin);
    }
}
