//! Runout choice status and compact, fully visible two-board presentation.
use super::{TexasTableLayout, add_texas_card};
use crate::app::presentation::{ACCENT, TEXT, add_text};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::{TexasPhaseView, TexasSnapshot};
use leocard_ui::{UiComposer, spawn_node};

pub(super) fn add_runout_status(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    assets: &UiAssets,
) {
    if !matches!(game.phase, TexasPhaseView::ChoosingRunout) {
        return;
    }
    let voter = game
        .players
        .iter()
        .find(|player| Some(player.player.id) == game.current_player);
    let status = if game.current_player == Some(game.you) {
        "请选择发一次或发两次".to_owned()
    } else {
        voter.map_or_else(
            || "等待选择".to_owned(),
            |player| format!("等待 {} 选择", player.player.name),
        )
    };
    let area = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(28),
            right: percent(28),
            bottom: px(TexasTableLayout::ACTION_BOTTOM + 56.0 * TexasTableLayout::ACTION_SCALE),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    add_text(commands, area, status, 17.0, ACCENT, assets);
    add_text(
        commands,
        area,
        "全员选两次才发两次 已有公共牌共享",
        13.0,
        TEXT,
        assets,
    );
}

pub(super) fn add_second_board(
    commands: &mut UiComposer,
    table: Entity,
    game: &TexasSnapshot,
    assets: &UiAssets,
) {
    // Both rows fit above the existing pot on desktop and mobile. Do not enlarge
    // the table or move seats/chip zones when the second run appears.
    let size = TexasTableLayout::RUNOUT_CARD_SIZE;
    let card_size = (size.x, size.y);
    let shared = game
        .community
        .iter()
        .zip(&game.second_community)
        .take_while(|(first, second)| first == second)
        .count();
    let area = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(TexasTableLayout::RUNOUT_TOP),
            width: px(360),
            height: px(152),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    commands.entity(area).insert((
        UiTransform::from_translation(Val2::px(-180.0, 0.0)),
        ZIndex(10),
    ));
    let context = add_text(
        commands,
        area,
        format!("共享前 {shared} 张公共牌"),
        12.0,
        TEXT,
        assets,
    );
    commands.entity(context).insert(Node {
        height: px(16),
        flex_shrink: 0.0,
        ..default()
    });
    for (label, board) in [
        ("第一次", &game.community),
        ("第二次", &game.second_community),
    ] {
        let row = spawn_node(
            commands,
            area,
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(5),
                flex_shrink: 0.0,
                ..default()
            },
            None,
        );
        let label = add_text(commands, row, label, 14.0, ACCENT, assets);
        commands.entity(label).insert(Node {
            width: px(TexasTableLayout::RUNOUT_LABEL_WIDTH),
            flex_shrink: 0.0,
            ..default()
        });
        for card in board.iter().copied() {
            add_texas_card(commands, row, card, card_size, None, assets);
        }
    }
}
