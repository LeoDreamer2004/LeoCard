use super::super::MinecraftGuessUiState;
use super::super::view::GuessTable;
use crate::app::presentation::{AnchoredPopup, MUTED, TEXT, UiComposer, add_text, spawn_node};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_minecraft_guess::GraphNodeView;

#[derive(Component)]
pub(in super::super) struct NodeHint(Entity);

#[derive(Component)]
pub(in super::super) struct NodeTooltip;

pub(super) fn add_node_tooltip(
    table: &GuessTable<'_>,
    commands: &mut UiComposer,
    layer: Entity,
    anchor: Entity,
    node: &GraphNodeView,
) {
    let popup = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            max_width: px(250),
            padding: UiRect::axes(px(12), px(9)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    commands.entity(popup).insert((
        table.game_assets.popup_frame(),
        NodeTooltip,
        AnchoredPopup(anchor),
        GlobalZIndex(1600),
        Visibility::Hidden,
        Pickable::IGNORE,
    ));
    let kind = node.kind.map_or("标签", |kind| kind.label());
    let title = add_text(commands, popup, kind, 13.0, TEXT, table.assets);
    commands.entity(title).insert(Pickable::IGNORE);
    if node.revealed() {
        let label = add_text(commands, popup, node.label(), 12.0, TEXT, table.assets);
        commands.entity(label).insert(Pickable::IGNORE);
    }
    if let Some(count) = node.character_count {
        let count = add_text(
            commands,
            popup,
            format!("{count} 字"),
            12.0,
            MUTED,
            table.assets,
        );
        commands.entity(count).insert(Pickable::IGNORE);
    }
    commands.entity(anchor).insert(NodeHint(popup));
}

pub(in super::super) fn sync_graph_hints(
    state: Res<MinecraftGuessUiState>,
    hints: Query<(&NodeHint, &PickingInteraction, &InheritedVisibility)>,
    mut popups: Query<(Entity, &mut Visibility), With<NodeTooltip>>,
) {
    let hovered = if state.graph.gesture.is_none() {
        hints.iter().find_map(|(hint, interaction, visible)| {
            (visible.get() && *interaction != PickingInteraction::None).then_some(hint.0)
        })
    } else {
        None
    };
    for (entity, mut visibility) in &mut popups {
        visibility.set_if_neq(if Some(entity) == hovered {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}
