use super::playback::{self, LinkPlayback, LinkTileVisual};
use super::state::MahjongLinkUiState;
use super::{MahjongLinkUiAction, controls, players, summary};
use crate::app::games::GameTableContext;
use crate::app::games::mahjong::{
    MahjongAssets, MahjongTileMaterial, attach_mahjong_tile_surface, mahjong_local_light,
};
use crate::app::presentation::{
    ACCENT, CardTableSurface, DESIGN_WIDTH, MUTED, TEXT, add_auto_play_overlay, add_text,
};
use crate::app::shell::{ChatPanelOptions, UiAction, add_reconnecting_overlay};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_mahjong_link::{BOARD_COLUMNS, BOARD_ROWS, LinkCell};
use leocard_protocol::MahjongLinkSnapshot;
use leocard_ui::{UiComposer, spawn_node};

pub(super) const TILE_WIDTH: f32 = 40.0;
pub(super) const TILE_HEIGHT: f32 = 52.0;
pub(super) const STEP_X: f32 = 44.0;
pub(super) const STEP_Y: f32 = 56.0;

pub(super) fn can_act(game: &MahjongLinkSnapshot) -> bool {
    !game.match_complete
        && game.current_player == Some(game.you)
        && game
            .players
            .iter()
            .any(|player| player.player.id == game.you && !player.auto_play)
}

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    game: &MahjongLinkSnapshot,
    ui: &MahjongLinkUiState,
    tiles: &MahjongAssets,
    playback: &LinkPlayback,
    mut context: GameTableContext<'_>,
) {
    let content = commands.keyed_child(
        root,
        format!("mahjong-link:{:?}", game.match_id),
        (Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            ..default()
        },),
    );
    CardTableSurface {
        brightness: context.form.table_brightness,
        custom_felt: context.appearance.cards.custom_felt.as_ref(),
    }
    .apply(commands, content, context.assets, context.table_materials);
    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(660),
            ..default()
        },
        None,
    );
    commands
        .entity(table)
        .insert(UiTransform::from_translation(Val2::px(
            -DESIGN_WIDTH * 0.5,
            0.0,
        )));
    players::render(commands, table, game, playback, &mut context);

    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(300),
            top: px(12),
            width: px(865),
            height: px(36),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    let current_name = game
        .current_player
        .and_then(|id| game.players.iter().find(|player| player.player.id == id))
        .map(|player| player.player.name.as_str())
        .unwrap_or("");
    add_text(
        commands,
        status,
        if game.match_complete {
            "本局结束".to_owned()
        } else if can_act(game) {
            if game.push_used {
                "已推动一次  请消除一对牌或跳过".to_owned()
            } else {
                "轮到你了  可直接消除或先推一次".to_owned()
            }
        } else {
            format!("等待 {current_name} 操作")
        },
        22.0,
        if can_act(game) { ACCENT } else { TEXT },
        context.assets,
    );
    add_text(
        commands,
        status,
        format!("剩余 {} 张", game.board.remaining()),
        18.0,
        MUTED,
        context.assets,
    );

    let board = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(365),
            top: px(66),
            width: px(STEP_X * BOARD_COLUMNS as f32),
            height: px(STEP_Y * BOARD_ROWS as f32),
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    for (index, tile) in game.board.cells().iter().enumerate() {
        let Some(kind) = tile else {
            continue;
        };
        let cell = LinkCell {
            row: (index / BOARD_COLUMNS) as u8,
            column: (index % BOARD_COLUMNS) as u8,
        };
        let selected = ui.selected == Some(cell);
        let node = commands.keyed_child(
            board,
            format!("tile:{index}"),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(f32::from(cell.column) * STEP_X),
                    top: px(f32::from(cell.row) * STEP_Y),
                    width: px(TILE_WIDTH),
                    height: px(TILE_HEIGHT),
                    ..default()
                },
                ZIndex(i32::from(cell.row)),
            ),
        );
        commands.entity(node).insert((
            UiTransform::IDENTITY,
            LinkTileVisual {
                offset: playback.tile_offset(cell),
                started: playback.started,
                selected,
            },
        ));
        if can_act(game) {
            // Retained tiles must undo the previous turn's explicit picking opt-out.
            commands.entity(node).insert((
                Button,
                Pickable::default(),
                UiAction::MahjongLink(MahjongLinkUiAction::Select(cell)),
            ));
        }
        let material = commands.material(
            node,
            context.mahjong_tile_materials,
            MahjongTileMaterial {
                params: Vec4::new(if selected { 1.0 } else { 0.0 }, 0.0, 1.0, -1.0),
                lighting: mahjong_local_light(0),
                highlight: if selected {
                    Vec4::new(1.0, 0.75, 0.15, 0.46)
                } else {
                    Vec4::ZERO
                },
                glyph: tiles.tiles[kind].clone(),
                height: tiles.tile_heights[kind].clone(),
                ..default()
            },
        );
        attach_mahjong_tile_surface(commands, node, material, false);
        // Decorative geometry must not steal the tile's touch/click target.
        if !can_act(game) {
            commands.entity(node).insert(Pickable::IGNORE);
        }
    }
    playback::path(commands, board, playback);
    controls::render(commands, table, game, ui, context.assets);
    summary::render(commands, table, game, &context);
    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you);
    let auto_play = (!game.match_complete)
        .then(|| own.map(|player| player.auto_play))
        .flatten();
    ChatPanelOptions {
        help: Some(UiAction::MahjongLink(MahjongLinkUiAction::ToggleGuide)),
        ..ChatPanelOptions::new(auto_play, &[])
    }
    .spawn(commands, content, context.chat, context.assets);
    if auto_play == Some(true) {
        add_auto_play_overlay(commands, content, context.assets);
    }
    if ui.guide_open {
        controls::guide(commands, content, context.assets);
    }
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, context.assets);
    }
}
