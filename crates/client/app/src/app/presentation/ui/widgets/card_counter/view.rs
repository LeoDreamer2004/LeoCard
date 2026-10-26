use super::CardCounterWindowState;
use crate::app::presentation::{MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::add_cozy_panel;
use bevy::picking::Pickable;
use bevy::{prelude::*, ui::RelativeCursorPosition};
use leocard_ui::{DraggableWindow, DraggableWindowOwner, UiComposer, WindowDragHandle, spawn_node};
use std::marker::PhantomData;

pub(crate) struct CardCounterCell {
    pub image: Handle<Image>,
    pub remaining: u8,
    pub color: Color,
}

pub(crate) fn add_card_counter_window<T: DraggableWindowOwner>(
    commands: &mut UiComposer,
    parent: Entity,
    state: &CardCounterWindowState,
    rows: [[Option<CardCounterCell>; 14]; 4],
    assets: &UiAssets,
) {
    let panel = add_cozy_panel(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(state.drag.position.x),
            top: px(state.drag.position.y),
            width: px(509.0),
            max_width: percent(94),
            padding: UiRect::all(px(13.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(5.0),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        DraggableWindow::<T>(PhantomData),
        GlobalZIndex(1400),
        Pickable::default(),
    ));
    let handle = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(20.0),
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands.entity(handle).insert((
        WindowDragHandle::<T>(PhantomData),
        RelativeCursorPosition::default(),
    ));
    let title = add_text(commands, handle, "记牌器", 13.0, TEXT, assets);
    commands.entity(title).insert(Pickable::IGNORE);
    for cells in rows {
        let row = spawn_node(
            commands,
            panel,
            Node {
                width: percent(100),
                column_gap: px(3.0),
                ..default()
            },
            None,
        );
        for cell in cells {
            let slot = spawn_node(
                commands,
                row,
                Node {
                    flex_basis: px(0),
                    flex_grow: 1.0,
                    min_width: px(0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(2.0),
                    ..default()
                },
                None,
            );
            commands.entity(slot).insert(Pickable::IGNORE);
            if let Some(cell) = cell {
                commands.child(
                    slot,
                    (
                        Node {
                            width: percent(100),
                            aspect_ratio: Some(48.0 / 65.0),
                            ..default()
                        },
                        ImageNode::new(cell.image),
                        Pickable::IGNORE,
                    ),
                );
                let number = add_text(
                    commands,
                    slot,
                    cell.remaining.to_string(),
                    15.0,
                    if cell.remaining == 0 {
                        MUTED.with_alpha(0.4)
                    } else {
                        cell.color
                    },
                    assets,
                );
                commands.entity(number).insert(Pickable::IGNORE);
            }
        }
    }
}
