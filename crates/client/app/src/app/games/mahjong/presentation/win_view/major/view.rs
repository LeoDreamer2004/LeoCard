use super::super::context::WinShowcaseContext;
use super::super::tile_motion::{MahjongShowcaseTile, ShowcaseTileStyle};
use super::motion::{MahjongMajorShowcase, MajorShowcaseGlyph};
use super::{MahjongMajorFanAssets, MahjongWinArtMaterial, mahjong_major_fan};
use crate::app::games::mahjong::{
    MahjongTileSize, MahjongWinTileSizes, render_mahjong_win_tile_row,
};
use crate::app::presentation::DESIGN_WIDTH;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::{UiComposer, spawn_node};

pub(in crate::app::games::mahjong::presentation::win_view) struct MajorShowcaseContext<'a, 'w, 's> {
    pub showcase: WinShowcaseContext<'a, 'w, 's>,
    pub assets: &'a MahjongMajorFanAssets,
    pub materials: &'a mut Assets<MahjongWinArtMaterial>,
}

pub(in crate::app::games::mahjong::presentation::win_view) fn render_major_win_showcase(
    context: MajorShowcaseContext<'_, '_, '_>,
) {
    let MajorShowcaseContext {
        showcase,
        assets: art_assets,
        materials: art_materials,
    } = context;
    let WinShowcaseContext {
        commands,
        viewport,
        game,
        winner,
        timing,
        game_assets,
        materials,
        ..
    } = showcase;
    let Some(player) = game
        .players
        .iter()
        .find(|player| player.player.id == winner.player)
    else {
        return;
    };
    if player.revealed_hand.is_none() {
        return;
    }
    let fan = mahjong_major_fan(winner)
        .expect("major showcase has an individual fan worth at least 48 points");
    let art = art_assets.get(fan);
    let root = layer(
        commands,
        viewport,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            overflow: Overflow::visible(),
            ..default()
        },
        0,
    );
    commands
        .entity(root)
        .insert((GlobalZIndex(1050), Visibility::Hidden));
    let backdrop = layer(commands, root, full_layer(), 0);
    commands
        .entity(backdrop)
        .insert(BackgroundColor(Color::NONE));
    let background = layer(commands, root, full_layer(), 1);
    let background_material = commands.material(
        background,
        art_materials,
        MahjongWinArtMaterial::background(art),
    );
    commands
        .entity(background)
        .insert(MaterialNode(background_material.clone()));
    let canvas = layer(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            width: px(DESIGN_WIDTH),
            height: percent(100),
            overflow: Overflow::visible(),
            ..default()
        },
        2,
    );
    commands
        .entity(canvas)
        .insert(UiTransform::from_translation(Val2::px(
            -DESIGN_WIDTH * 0.5,
            0.0,
        )));
    let rect = art.spec.title_rect;
    let title = layer(
        commands,
        canvas,
        Node {
            position_type: PositionType::Absolute,
            left: percent(rect.x * 100.0),
            top: percent(rect.y * 100.0),
            width: percent(rect.z * 100.0),
            height: percent(rect.w * 100.0),
            overflow: Overflow::visible(),
            ..default()
        },
        2,
    );
    let mut glyphs = Vec::with_capacity(art.spec.glyphs.len());
    for (index, &bounds) in art.spec.glyphs.iter().enumerate() {
        let glyph = layer(commands, title, full_layer(), index as i32);
        let material = commands.material(
            glyph,
            art_materials,
            MahjongWinArtMaterial::glyph(art, bounds),
        );
        commands
            .entity(glyph)
            .insert(MaterialNode(material.clone()));
        glyphs.push(MajorShowcaseGlyph {
            entity: glyph,
            material,
            bounds,
        });
    }
    let hand_rect = art.spec.hand_rect;
    let mut row_node = Node {
        position_type: PositionType::Absolute,
        left: percent((hand_rect.x + hand_rect.z * 0.5) * 100.0),
        top: percent((hand_rect.y + hand_rect.w * 0.5) * 100.0),
        height: px(86),
        align_items: AlignItems::FlexEnd,
        overflow: Overflow::visible(),
        ..default()
    };
    let row = layer(commands, canvas, row_node.clone(), 3);
    let hand = render_mahjong_win_tile_row(
        commands,
        row,
        player,
        winner,
        MahjongWinTileSizes {
            meld: MahjongTileSize::MajorShowcase,
            hand: MahjongTileSize::MajorShowcase,
        },
        game_assets,
        materials,
    );
    let scale = (DESIGN_WIDTH * hand_rect.z / hand.width).min(1.0);
    row_node.width = px(hand.width);
    commands.entity(row).insert((
        row_node,
        UiTransform {
            translation: Val2::px(-hand.width * 0.5, -43.0),
            scale: Vec2::splat(scale),
            ..default()
        },
    ));
    for (index, &tile) in hand.tiles.iter().enumerate() {
        commands.entity(tile).insert((
            Visibility::Hidden,
            MahjongShowcaseTile::new(timing, ShowcaseTileStyle::Major, index, hand.tiles.len()),
        ));
    }
    commands.entity(root).insert(MahjongMajorShowcase {
        timing,
        viewport,
        backdrop,
        background: background_material,
        glyphs,
    });
}

fn full_layer() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

fn layer(commands: &mut UiComposer, parent: Entity, node: Node, z_index: i32) -> Entity {
    let entity = spawn_node(commands, parent, node, None);
    commands.entity(entity).insert((
        ZIndex(z_index),
        Pickable::IGNORE,
        Visibility::Inherited,
        UiTransform::IDENTITY,
    ));
    entity
}
