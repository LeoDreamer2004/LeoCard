use super::layout::{self, TILE_SCALE};
use super::playback::{self, LinkPlayback, LinkTileVisual};
use super::state::MahjongLinkUiState;
use super::{MahjongLinkUiAction, controls, players, summary};
use crate::app::games::GameTableContext;
use crate::app::presentation::mahjong::{
    MahjongProjectionViewport, MahjongTableRoot, MahjongTileAssets, MahjongTileGeometry,
    MahjongTileMaterial, WORLD_SCALE, attach_mahjong_tile_surface, mahjong_local_light,
};
use crate::app::presentation::{
    ACCENT, DESIGN_WIDTH, MUTED, TEXT, TableBackground, TableBackgroundMaterial, TableSurfaceKind,
    add_auto_play_overlay, add_text,
};
use crate::app::shell::{ChatPanelOptions, UiAction, add_reconnecting_overlay};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_mahjong_link::{BOARD_COLUMNS, LinkCell};
use leocard_protocol::MahjongLinkSnapshot;
use leocard_ui::{UiComposer, spawn_node};

#[derive(Component)]
pub(super) struct LinkTileHitbox;

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
    tiles: &MahjongTileAssets,
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
    let material = commands.material(
        content,
        context.table_materials,
        TableBackgroundMaterial::new(
            TableSurfaceKind::Mahjong,
            context.form.mahjong_table_brightness,
            &tiles.table,
            context.appearance.mahjong.custom_felt.as_ref(),
        ),
    );
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands.entity(table).insert((
        MahjongTableRoot,
        UiTransform::from_translation(Val2::px(-DESIGN_WIDTH * 0.5, 0.0)),
    ));
    players::render(commands, table, game, playback, &mut context);

    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(220),
            right: px(220),
            top: px(22),
            height: px(36),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    commands.entity(status).insert(ZIndex(100));
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
        if game.consecutive_passes > 0 {
            format!(
                "剩余 {} 张  连续跳过 {} 人",
                game.board.remaining(),
                game.consecutive_passes
            )
        } else {
            format!("剩余 {} 张", game.board.remaining())
        },
        18.0,
        MUTED,
        context.assets,
    );

    let viewport = MahjongProjectionViewport::default().size;
    for (index, tile) in game.board.cells().iter().enumerate() {
        let Some(kind) = tile else {
            continue;
        };
        let cell = LinkCell {
            row: (index / BOARD_COLUMNS) as u8,
            column: (index % BOARD_COLUMNS) as u8,
        };
        let selected = ui.selected == Some(cell);
        let hitbox = commands.keyed_child(
            table,
            format!("hitbox:{index}"),
            (
                layout::hitbox_node(layout::tile_bounds(cell, Vec2::ZERO, 0.0, viewport)),
                LinkTileHitbox,
                ZIndex(20 + i32::from(cell.row) * 32 + 8 - (i32::from(cell.column) - 8).abs()),
            ),
        );
        if can_act(game) {
            commands.entity(hitbox).insert((
                Button,
                Pickable::default(),
                UiAction::MahjongLink(MahjongLinkUiAction::Select(cell)),
            ));
        } else {
            commands.entity(hitbox).insert(Pickable::IGNORE);
        }
        let center = layout::cell_layout(cell);
        let node = commands.keyed_child(
            table,
            format!("tile:{index}"),
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(center.x - WORLD_SCALE * 0.5),
                    top: px(center.y - WORLD_SCALE / MahjongTileGeometry::FACE_ASPECT_RATIO * 0.5),
                    width: px(WORLD_SCALE),
                    height: px(WORLD_SCALE / MahjongTileGeometry::FACE_ASPECT_RATIO),
                    ..default()
                },
                Pickable::IGNORE,
            ),
        );
        commands.entity(node).insert((
            UiTransform::from_scale(Vec2::splat(TILE_SCALE)),
            LinkTileVisual {
                cell,
                hitbox,
                offset: playback.tile_offset(cell),
                started: playback.started,
                selected,
            },
        ));
        let material = commands.material(
            node,
            context.mahjong_tile_materials,
            MahjongTileMaterial {
                table_projection: true,
                params: Vec4::new(if selected { 1.0 } else { 0.0 }, 0.0, 1.0, 0.0),
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
        attach_mahjong_tile_surface(commands, node, material, true);
    }
    playback::path(commands, table, playback);
    controls::render(commands, table, game, ui, context.assets);
    summary::render(commands, table, game, &context);
    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you);
    let auto_play = (!game.match_complete)
        .then(|| own.map(|player| player.auto_play))
        .flatten();
    ChatPanelOptions::new(auto_play, &[]).spawn(commands, content, context.chat, context.assets);
    if auto_play == Some(true) {
        add_auto_play_overlay(commands, content, context.assets);
    }
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, context.assets);
    }
}
