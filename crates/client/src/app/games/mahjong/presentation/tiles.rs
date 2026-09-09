use super::claim::mahjong_claim_held_tile_visual;
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_MELD_WIDTH, MAHJONG_REMOTE_MELD_WIDTH,
    MahjongAssets, MahjongClaimHeldTile, MahjongDealSpec, MahjongDealTile, MahjongTileMaterial,
    mahjong_local_light, mahjong_local_shadow,
};
use crate::app::presentation::spawn_node;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::{
    MahjongClaim, MahjongKongKind, MahjongMeldKind, MahjongTileKind, MahjongWind,
};
use leocard_protocol::MahjongPublicMeldView;

#[derive(Clone, Copy)]
pub(crate) enum MahjongTileSize {
    River,
    Mini,
    OwnMeld,
    HiddenSide,
    HiddenOpposite,
}

pub(crate) struct MahjongTileVisual {
    pub kind: Option<MahjongTileKind>,
    pub size: MahjongTileSize,
    pub index: usize,
    pub highlighted: bool,
    pub deal: Option<MahjongDealSpec>,
    pub relative: u8,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn render_mahjong_staged_meld(
    commands: &mut Commands,
    parent: Entity,
    claim: &ActiveMahjongClaimPresentation,
    index: &mut usize,
    relative: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let Some(claimed_tile) = claim.tile else {
        return;
    };
    let claimed_kind = claimed_tile.kind();
    let base = match claim.claim {
        MahjongClaim::Chow { start } => {
            let MahjongTileKind::Suited { suit, rank } = claimed_kind else {
                return;
            };
            let mut base = [
                Some(MahjongTileKind::suited(suit, start)),
                Some(MahjongTileKind::suited(suit, start + 1)),
                Some(MahjongTileKind::suited(suit, start + 2)),
            ];
            base[usize::from(rank - start)] = None;
            base
        }
        MahjongClaim::Pung => [Some(claimed_kind), None, Some(claimed_kind)],
        MahjongClaim::Kong => [Some(claimed_kind); 3],
        MahjongClaim::Pass | MahjongClaim::Win => return,
    };
    let own_meld = relative == 0;
    let tile_size = if own_meld {
        MahjongTileSize::OwnMeld
    } else {
        MahjongTileSize::Mini
    };
    let (group_width, group_height, tile_advance) = if own_meld {
        (MAHJONG_OWN_MELD_WIDTH, 80.0, 45.0)
    } else {
        (MAHJONG_REMOTE_MELD_WIDTH, 54.0, 24.0)
    };
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            min_width: px(group_width),
            height: px(group_height),
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let (scale_x, offset_y) = mahjong_claim_held_tile_visual(claim.elapsed);
    for kind in base {
        if let Some(kind) = kind {
            let tile = add_mahjong_tile_material(
                commands,
                group,
                MahjongTileVisual {
                    kind: Some(kind),
                    size: tile_size,
                    index: *index,
                    highlighted: false,
                    deal: None,
                    relative,
                },
                assets,
                materials,
            );
            commands.entity(tile).insert((
                MahjongClaimHeldTile {
                    player: claim.player,
                },
                UiTransform {
                    translation: Val2::px(0.0, offset_y),
                    scale: Vec2::new(scale_x, 1.0),
                    ..default()
                },
            ));
        } else {
            let slot = spawn_node(
                commands,
                group,
                Node {
                    width: px(tile_advance),
                    min_width: px(tile_advance),
                    height: px(1),
                    ..default()
                },
                None,
            );
            commands.entity(slot).insert(FocusPolicy::Pass);
        }
        *index += 1;
    }
}

pub(crate) fn render_mahjong_meld(
    commands: &mut Commands,
    parent: Entity,
    meld: &MahjongPublicMeldView,
    index: &mut usize,
    relative: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let (base, stacked) = match (meld.kind, meld.tile) {
        (MahjongMeldKind::Chow, Some(MahjongTileKind::Suited { suit, rank })) => (
            vec![
                Some(MahjongTileKind::suited(suit, rank)),
                Some(MahjongTileKind::suited(suit, rank + 1)),
                Some(MahjongTileKind::suited(suit, rank + 2)),
            ],
            None,
        ),
        (MahjongMeldKind::Pung, Some(kind)) => (vec![Some(kind); 3], None),
        (MahjongMeldKind::Kong(_), Some(kind)) => (vec![Some(kind); 3], Some(Some(kind))),
        (MahjongMeldKind::Kong(MahjongKongKind::Concealed), None) => (vec![None; 3], Some(None)),
        _ => return,
    };
    let own_meld = relative == 0;
    let tile_size = if own_meld {
        MahjongTileSize::OwnMeld
    } else {
        MahjongTileSize::Mini
    };
    let (group_width, group_height, stack_left, stack_top, stack_width, stack_height) = if own_meld
    {
        (MAHJONG_OWN_MELD_WIDTH, 80.0, 45.0, 7.0, 50.0, 68.0)
    } else {
        (MAHJONG_REMOTE_MELD_WIDTH, 54.0, 24.0, 12.0, 27.0, 37.0)
    };
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            min_width: px(group_width),
            height: px(group_height),
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    for kind in base {
        add_mahjong_tile_material(
            commands,
            group,
            MahjongTileVisual {
                kind,
                size: tile_size,
                index: *index,
                highlighted: false,
                deal: None,
                relative,
            },
            assets,
            materials,
        );
        *index += 1;
    }
    if let Some(kind) = stacked {
        let holder = spawn_node(
            commands,
            group,
            Node {
                position_type: PositionType::Absolute,
                left: px(stack_left),
                top: px(stack_top),
                width: px(stack_width),
                height: px(stack_height),
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(holder).insert(ZIndex(100 + *index as i32));
        let stacked_tile = add_mahjong_tile_material(
            commands,
            holder,
            MahjongTileVisual {
                kind,
                size: tile_size,
                index: *index,
                highlighted: false,
                deal: None,
                relative,
            },
            assets,
            materials,
        );
        let shadow = mahjong_local_shadow(relative) * 1.4;
        commands.entity(stacked_tile).insert(BoxShadow::new(
            Color::BLACK.with_alpha(0.24),
            px(shadow.x),
            px(shadow.y),
            px(0),
            px(4),
        ));
        *index += 1;
    }
}

pub(crate) fn add_mahjong_tile_material(
    commands: &mut Commands,
    parent: Entity,
    visual: MahjongTileVisual,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> Entity {
    let MahjongTileVisual {
        kind,
        size,
        index,
        highlighted,
        deal,
        relative,
    } = visual;
    let (width, height, overlap) = match size {
        MahjongTileSize::River => (33.0, 45.0, -3.0),
        MahjongTileSize::Mini => (27.0, 37.0, -3.0),
        MahjongTileSize::OwnMeld => (50.0, 68.0, -5.0),
        MahjongTileSize::HiddenSide => (34.0, 46.0, -5.0),
        MahjongTileSize::HiddenOpposite => (31.0, 42.0, -4.0),
    };
    let back = kind.is_none();
    let glyph = kind.map_or_else(
        || game_assets.tile_back.clone(),
        |kind| {
            game_assets
                .tiles
                .get(&kind)
                .cloned()
                .expect("所有麻将牌面都应预加载")
        },
    );
    let height_texture = kind.map_or_else(
        || game_assets.tile_back.clone(),
        |kind| {
            game_assets
                .tile_heights
                .get(&kind)
                .cloned()
                .expect("所有麻将凹刻高度图都应预加载")
        },
    );
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(
            0.0,
            if back { 1.0 } else { 0.0 },
            if deal.is_some() { 0.0 } else { 1.0 },
            match size {
                MahjongTileSize::HiddenSide => -2.0,
                MahjongTileSize::HiddenOpposite => -3.0,
                MahjongTileSize::OwnMeld => -4.0,
                MahjongTileSize::River | MahjongTileSize::Mini => 0.0,
            },
        ),
        lighting: mahjong_local_light(relative),
        glyph,
        height: height_texture,
    });
    let node = Node {
        width: px(width),
        height: px(height),
        min_width: px(width),
        margin: UiRect::right(px(overlap)),
        border: UiRect::all(px(if highlighted { 1 } else { 0 })),
        border_radius: BorderRadius::all(px(3)),
        overflow: Overflow::visible(),
        ..default()
    };
    let final_rotation = 0.0;
    let final_offset = Vec2::ZERO;
    let final_shadow_alpha = match size {
        MahjongTileSize::River => 0.12,
        MahjongTileSize::Mini => 0.08,
        MahjongTileSize::OwnMeld => 0.12,
        MahjongTileSize::HiddenSide | MahjongTileSize::HiddenOpposite => 0.14,
    };
    let shadow = mahjong_local_shadow(relative);
    let entity = commands
        .spawn((
            node,
            MaterialNode(material),
            BorderColor::all(if highlighted {
                Color::srgba(0.96, 0.78, 0.28, 0.90)
            } else {
                Color::NONE
            }),
            BoxShadow::new(
                if highlighted {
                    Color::srgba(0.95, 0.72, 0.20, 0.35)
                } else {
                    Color::BLACK.with_alpha(if deal.is_some() {
                        0.0
                    } else {
                        final_shadow_alpha
                    })
                },
                px(shadow.x),
                px(shadow.y),
                px(0),
                px(if highlighted { 5 } else { 2 }),
            ),
            ZIndex(index as i32),
            UiTransform {
                translation: Val2::px(final_offset.x, final_offset.y),
                rotation: Rot2::radians(final_rotation),
                ..default()
            },
        ))
        .id();
    if let Some(deal) = deal {
        commands.entity(entity).insert(MahjongDealTile {
            elapsed: 0.0,
            start_offset: deal.start_offset,
            start_rotation: deal.start_rotation,
            final_offset,
            final_rotation,
            final_shadow_alpha,
        });
    }
    commands.entity(parent).add_child(entity);
    entity
}

pub(crate) fn wind_label(wind: MahjongWind) -> &'static str {
    match wind {
        MahjongWind::East => "东",
        MahjongWind::South => "南",
        MahjongWind::West => "西",
        MahjongWind::North => "北",
    }
}
