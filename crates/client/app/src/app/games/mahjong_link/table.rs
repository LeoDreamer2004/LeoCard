use super::input::{LinkPushRange, LinkTileHitbox};
use super::layout::{self, TILE_SCALE};
use super::playback::{self, LinkPlayback, LinkTileVisual};
use super::state::can_act;
use super::{MahjongLinkUiAction, players, summary};
use crate::app::games::GameTableContext;
use crate::app::presentation::mahjong::{
    MahjongProjectionViewport, MahjongTableRoot, MahjongTileAssets, MahjongTileGeometry,
    MahjongTileMaterial, WORLD_SCALE, attach_mahjong_tile_surface, mahjong_local_light,
};
use crate::app::presentation::{
    DESIGN_WIDTH, TableBackground, TableBackgroundMaterial, TableSurfaceKind, add_auto_play_overlay,
};
use crate::app::shell::{
    ChatAuxiliaryAction, ChatPanelOptions, UiAction, add_reconnecting_overlay,
};
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_mahjong_link::{BOARD_COLUMNS, LinkCell};
use leocard_protocol::MahjongLinkSnapshot;
use leocard_ui::{TouchDragTarget, UiComposer, spawn_node};

pub(super) fn render(
    commands: &mut UiComposer,
    root: Entity,
    game: &MahjongLinkSnapshot,
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
            &context.assets.tables,
            &context.assets.tables.felts.mahjong,
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

    let enabled = can_act(game);
    let viewport = MahjongProjectionViewport::default().size;
    for (index, tile) in game.board.cells().iter().enumerate() {
        let Some(kind) = tile else {
            continue;
        };
        let cell = LinkCell {
            row: (index / BOARD_COLUMNS) as u8,
            column: (index % BOARD_COLUMNS) as u8,
        };
        let movable = enabled && !game.push_used && LinkPushRange::at(&game.board, cell).is_some();
        let hitbox = commands.keyed_child(
            table,
            format!("hitbox:{index}"),
            (
                layout::hitbox_node(layout::tile_bounds(cell, Vec2::ZERO, 0.0, viewport)),
                LinkTileHitbox(cell),
                ZIndex(20 + i32::from(cell.row) * 32 + 8 - (i32::from(cell.column) - 8).abs()),
            ),
        );
        if enabled {
            commands.entity(hitbox).insert((
                Pickable::default(),
                PickingInteraction::default(),
                TouchDragTarget,
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
                movable,
            },
        ));
        let material = commands.material(
            node,
            context.mahjong_tile_materials,
            MahjongTileMaterial {
                table_projection: true,
                params: Vec4::Z,
                lighting: mahjong_local_light(0),
                glyph: tiles.tiles[kind].clone(),
                height: tiles.tile_heights[kind].clone(),
                ..default()
            },
        );
        attach_mahjong_tile_surface(commands, node, material, true);
    }
    playback::path(commands, table, playback);
    summary::render(commands, table, game, &context);
    let own = game
        .players
        .iter()
        .find(|player| player.player.id == game.you);
    let auto_play = (!game.match_complete)
        .then(|| own.map(|player| player.auto_play))
        .flatten();
    let pass = [ChatAuxiliaryAction {
        label: "过",
        action: enabled.then_some(UiAction::MahjongLink(MahjongLinkUiAction::Pass)),
        highlighted: false,
    }];
    let auxiliary = if game.match_complete { &[][..] } else { &pass };
    ChatPanelOptions::new(auto_play, auxiliary).spawn(
        commands,
        content,
        context.chat,
        context.assets,
    );
    if auto_play == Some(true) {
        add_auto_play_overlay(commands, content, context.assets);
    }
    if let Some(message) = context.client.0.reconnecting_message() {
        add_reconnecting_overlay(commands, content, message, context.assets);
    }
}
