use super::super::{
    MahjongAssets, MahjongTileHighlight, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    add_mahjong_tile_material,
};
use super::{
    MahjongReadyHint, MahjongReadyHintState, MahjongWaitPopupLink, mahjong_current_waits,
    waits::MahjongWait,
};
use crate::app::presentation::{ACCENT, MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::picking::{Pickable, hover::PickingInteraction};
use bevy::prelude::*;
use leocard_mahjong::MahjongScoreResult;
use leocard_protocol::MahjongSnapshot;

pub(in super::super) fn add_mahjong_wait_popup(
    commands: &mut Commands,
    hand_tile: Entity,
    waits: &[MahjongWait],
    initially_visible: bool,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
    assets: &UiAssets,
) {
    let (_, width) = popup_dimensions(waits);
    if let Some(popup) = add_wait_popup(
        commands,
        hand_tile,
        waits,
        Node {
            left: px((50.0 - width) * 0.5),
            bottom: px(76),
            ..default()
        },
        game_assets,
        materials,
        assets,
    ) {
        commands.entity(popup).insert(if initially_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}

pub(in super::super) fn add_mahjong_ready_hint(
    commands: &mut Commands,
    chat_panel: Entity,
    game: &MahjongSnapshot,
    state: MahjongReadyHintState,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
    assets: &UiAssets,
) {
    let waits = mahjong_current_waits(game, state.show_fans);
    if waits.is_empty() {
        return;
    }
    let icon = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(272),
                width: px(32),
                height: px(32),
                ..default()
            },
            ImageNode::new(game_assets.ready_hand.clone()),
            GlobalZIndex(2000),
            Pickable::default(),
            MahjongReadyHint,
            if state.hovered {
                PickingInteraction::Hovered
            } else {
                PickingInteraction::None
            },
        ))
        .id();
    commands.entity(chat_panel).add_child(icon);
    if let Some(popup) = add_wait_popup(
        commands,
        icon,
        &waits,
        Node {
            right: px(38),
            bottom: px(0),
            ..default()
        },
        game_assets,
        materials,
        assets,
    ) {
        commands.entity(popup).insert(if state.hovered {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    }
}

fn add_wait_popup(
    commands: &mut Commands,
    anchor: Entity,
    waits: &[MahjongWait],
    position: Node,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
    assets: &UiAssets,
) -> Option<Entity> {
    if waits.is_empty() {
        return None;
    }
    let (cell_width, width) = popup_dimensions(waits);
    let popup = spawn_node(
        commands,
        anchor,
        Node {
            position_type: PositionType::Absolute,
            width: px(width),
            padding: UiRect::all(px(7)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(8)),
            ..position
        },
        Some(Color::srgb(0.07, 0.18, 0.23).with_alpha(0.96)),
    );
    commands.entity(popup).insert((
        Visibility::Hidden,
        GlobalZIndex(2100),
        Pickable::IGNORE,
        BoxShadow::new(Color::BLACK.with_alpha(0.4), px(1), px(5), px(0), px(9)),
    ));
    commands.entity(anchor).insert(MahjongWaitPopupLink(popup));
    for (index, wait) in waits.iter().enumerate() {
        let cell = spawn_node(
            commands,
            popup,
            Node {
                width: px(cell_width),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(2),
                ..default()
            },
            None,
        );
        commands.entity(cell).insert(Pickable::IGNORE);
        add_mahjong_tile_material(
            commands,
            cell,
            MahjongTileVisual {
                kind: Some(wait.kind),
                size: MahjongTileSize::Mini,
                index,
                highlight: MahjongTileHighlight::None,
                deal: None,
                relative: 0,
            },
            game_assets,
            materials,
        );
        let count = add_text(
            commands,
            cell,
            wait.remaining.to_string(),
            13.0,
            if wait.remaining == 0 { MUTED } else { TEXT },
            assets,
        );
        commands.entity(count).insert(Pickable::IGNORE);
        if let Some(scores) = &wait.scores {
            add_wait_score(commands, cell, "自摸", &scores.self_draw, assets);
            add_wait_score(commands, cell, "和牌", &scores.discard, assets);
        }
    }
    Some(popup)
}

fn popup_dimensions(waits: &[MahjongWait]) -> (f32, f32) {
    let show_fans = waits.iter().any(|wait| wait.scores.is_some());
    let cell_width = if show_fans { 96.0 } else { 31.0 };
    let columns = waits.len().min(if show_fans { 6 } else { 10 });
    (cell_width, columns as f32 * cell_width + 14.0)
}

fn add_wait_score(
    commands: &mut Commands,
    cell: Entity,
    label: &str,
    score: &MahjongScoreResult,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        cell,
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(2),
            ..default()
        },
        None,
    );
    commands.entity(row).insert(Pickable::IGNORE);
    for (text, color) in [
        (label.to_owned(), MUTED),
        (
            score.points_without_flowers.to_string(),
            if score.points_without_flowers >= 8 {
                ACCENT
            } else {
                MUTED
            },
        ),
        (format!("+ {}", score.flower_points), MUTED),
        ("番".to_owned(), MUTED),
    ] {
        let text = add_text(commands, row, text, 11.0, color, assets);
        commands.entity(text).insert(Pickable::IGNORE);
    }
}
