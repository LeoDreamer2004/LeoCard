use crate::app::presentation::{BORDER, MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::connection::rooms::{
    JoinRoomAssets, JoinRoomUiState, RoomSelection, RoomSignal,
};
use crate::app::shell::{ConnectionUiAction, UiAction};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_ui::{CustomButtonMotion, UiComposer, spawn_node};

pub(super) struct RoomRow<'a> {
    pub selection: RoomSelection,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub address: &'a str,
    pub status: &'a str,
    pub color: Color,
    pub avatar_name: &'a str,
    pub avatar: Option<&'a Handle<Image>>,
    pub signal: RoomSignal,
}

pub(super) fn entry(
    commands: &mut UiComposer,
    parent: Entity,
    room: RoomRow<'_>,
    state: &JoinRoomUiState,
    assets: &UiAssets,
    skin: &JoinRoomAssets,
) {
    let selected = state.selected.as_ref() == Some(&room.selection);
    let key = match &room.selection {
        RoomSelection::Saved(address) => format!("saved:{address}"),
        RoomSelection::Lan(id) => format!("lan:{id}"),
    };
    let entry = commands.keyed_child(
        parent,
        key,
        (
            Button,
            CustomButtonMotion,
            UiTransform::IDENTITY,
            UiAction::Connection(ConnectionUiAction::SelectRoom(room.selection)),
            Node {
                width: percent(100),
                min_height: px(80),
                flex_shrink: 0.0,
                border: UiRect::all(px(2)),
                padding: UiRect::all(px(4)),
                align_items: AlignItems::Center,
                column_gap: px(14),
                ..default()
            },
            BorderColor::all(if selected { TEXT } else { Color::NONE }),
            BackgroundColor(if selected {
                Color::srgba(0.1, 0.1, 0.12, 0.7)
            } else {
                Color::NONE
            }),
        ),
    );
    let avatar = commands.child(
        entry,
        (
            Node {
                width: px(64),
                height: px(64),
                min_width: px(64),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderColor::all(BORDER),
            Pickable::IGNORE,
        ),
    );
    if let Some(image) = room.avatar {
        commands
            .entity(avatar)
            .insert(ImageNode::new(image.clone()));
    } else {
        label(
            commands,
            avatar,
            &room.avatar_name.chars().next().unwrap_or('玩').to_string(),
            28.0,
            MUTED,
            assets,
        );
    }

    let labels = spawn_node(
        commands,
        entry,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0),
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            overflow: Overflow::clip_x(),
            ..default()
        },
        None,
    );
    commands.entity(labels).insert(Pickable::IGNORE);
    label(commands, labels, room.title, 19.0, TEXT, assets);
    label(commands, labels, room.subtitle, 13.0, MUTED, assets);
    label(commands, labels, room.address, 12.0, MUTED, assets);
    let status = spawn_node(
        commands,
        entry,
        Node {
            width: px(110),
            flex_shrink: 0.0,
            align_self: AlignSelf::FlexStart,
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Column,
            row_gap: px(5),
            padding: UiRect::top(px(3)),
            ..default()
        },
        None,
    );
    commands.entity(status).insert(Pickable::IGNORE);
    commands.child(
        status,
        (
            Node {
                width: px(25),
                height: px(20),
                ..default()
            },
            ImageNode::new(skin.signal(room.signal, 0.0)),
            room.signal,
            Pickable::IGNORE,
        ),
    );
    if let RoomSignal::Online(ms) = room.signal {
        label(commands, status, &format!("{ms} ms"), 12.0, MUTED, assets);
    }
    label(commands, status, room.status, 12.0, room.color, assets);
}

pub(super) fn label(
    commands: &mut UiComposer,
    parent: Entity,
    text: &str,
    size: f32,
    color: Color,
    assets: &UiAssets,
) -> Entity {
    let entity = add_text(commands, parent, text, size, color, assets);
    commands
        .entity(entity)
        .insert((Pickable::IGNORE, TextLayout::no_wrap()));
    entity
}
