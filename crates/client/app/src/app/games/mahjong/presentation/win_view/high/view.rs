use super::super::context::WinShowcaseContext;
use super::super::tile_motion::{MahjongShowcaseTile, ShowcaseTileStyle};
use super::MahjongWinInkMaterial;
use super::motion::MahjongWinShowcase;
use crate::app::games::mahjong::{
    MahjongTileSize, MahjongWinTileSizes, render_mahjong_win_tile_row,
};
use crate::app::presentation::add_text;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::LineBreak;
use leocard_ui::{UiComposer, spawn_node};

pub(in crate::app::games::mahjong::presentation::win_view) fn render_high_win_showcase(
    context: WinShowcaseContext<'_, '_, '_>,
    ink_materials: &mut Assets<MahjongWinInkMaterial>,
) {
    let WinShowcaseContext {
        commands,
        table,
        game,
        winner,
        timing,
        assets,
        game_assets,
        materials,
        ..
    } = context;
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
    let root = layer(
        commands,
        table,
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
    let backdrop = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        Some(Color::NONE),
    );
    commands
        .entity(backdrop)
        .insert((ZIndex(0), Pickable::IGNORE));
    let canvas = layer(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(930),
            height: px(275),
            overflow: Overflow::visible(),
            ..default()
        },
        1,
    );
    commands
        .entity(canvas)
        .insert(UiTransform::from_translation(Val2::px(-465.0, -135.0)));
    let ink = layer(
        commands,
        canvas,
        Node {
            position_type: PositionType::Absolute,
            left: px(13),
            width: px(930),
            height: px(275),
            ..default()
        },
        0,
    );
    let ink_material = commands.material(
        ink,
        ink_materials,
        MahjongWinInkMaterial::new(&game_assets.win_showcase),
    );
    commands
        .entity(ink)
        .insert(MaterialNode(ink_material.clone()));
    let title = layer(
        commands,
        canvas,
        Node {
            position_type: PositionType::Absolute,
            left: px(73),
            top: px(8),
            width: px(165),
            height: px(91),
            ..default()
        },
        2,
    );
    commands.entity(title).insert(ImageNode {
        color: Color::NONE,
        ..ImageNode::new(game_assets.win_showcase.title.clone())
    });
    let name_holder = layer(
        commands,
        canvas,
        Node {
            position_type: PositionType::Absolute,
            left: px(35),
            top: px(86),
            width: px(240),
            height: px(26),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        2,
    );
    let name = add_text(
        commands,
        name_holder,
        &player.player.name,
        18.0,
        Color::srgb(0.93, 0.89, 0.72).with_alpha(0.0),
        assets,
    );
    commands.entity(name).insert((
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Pickable::IGNORE,
    ));
    let row = layer(
        commands,
        canvas,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(109),
            height: px(73),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            overflow: Overflow::visible(),
            ..default()
        },
        3,
    );
    let hand = render_mahjong_win_tile_row(
        commands,
        row,
        player,
        winner,
        MahjongWinTileSizes {
            meld: MahjongTileSize::InkShowcase,
            hand: MahjongTileSize::InkShowcase,
        },
        game_assets,
        materials,
    );
    // Four kongs can contain eighteen tiles. Keep them centered within the same brush stroke.
    commands
        .entity(row)
        .insert(UiTransform::from_scale(Vec2::splat(
            (780.0 / hand.width).min(1.0),
        )));
    for (index, &tile) in hand.tiles.iter().enumerate() {
        commands.entity(tile).insert((
            Visibility::Hidden,
            MahjongShowcaseTile::new(timing, ShowcaseTileStyle::Ink, index, hand.tiles.len()),
        ));
    }
    commands.entity(root).insert(MahjongWinShowcase {
        reveal_duration: timing.reveal_duration,
        start: timing.start,
        backdrop,
        ink,
        ink_material,
        title,
        name,
    });
}

fn layer(commands: &mut UiComposer, parent: Entity, node: Node, z_index: i32) -> Entity {
    let entity = spawn_node(commands, parent, node, None);
    commands
        .entity(entity)
        .insert((ZIndex(z_index), Pickable::IGNORE));
    entity
}
