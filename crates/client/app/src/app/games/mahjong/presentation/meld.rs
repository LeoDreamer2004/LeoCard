use super::super::layout::MahjongLayout as Layout;
use super::super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_MELD_WIDTH, MAHJONG_REMOTE_MELD_WIDTH,
    MahjongAssets, MahjongClaimHeldTile, MahjongTileHighlight, MahjongTileMaterial,
    mahjong_claim_landing_time,
};
use super::claim::{MahjongClaimLandingSlot, mahjong_claim_held_tile_visual};
use super::{
    MAHJONG_KONG_STACK_LIFT, MahjongTileSize, MahjongTileVisual, add_mahjong_tile_material,
};
use bevy::prelude::*;
use leocard_mahjong::{
    MahjongClaim, MahjongKongKind, MahjongMeldKind, MahjongPublicMeld, MahjongTileKind,
};
use leocard_ui::{UiComposer, spawn_node};

pub(crate) struct MahjongMeldPresentation<'a> {
    pub relative: u8,
    pub claim: Option<&'a ActiveMahjongClaimPresentation>,
}

pub(crate) fn render_mahjong_meld(
    commands: &mut UiComposer,
    parent: Entity,
    meld: &MahjongPublicMeld,
    index: &mut usize,
    presentation: MahjongMeldPresentation<'_>,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let (base, stacked) = match (meld.kind, meld.tile) {
        (MahjongMeldKind::Chow, Some(MahjongTileKind::Suited { suit, rank })) => (
            [
                Some(MahjongTileKind::suited(suit, rank)),
                Some(MahjongTileKind::suited(suit, rank + 1)),
                Some(MahjongTileKind::suited(suit, rank + 2)),
            ],
            None,
        ),
        (MahjongMeldKind::Pung, Some(kind)) => ([Some(kind); 3], None),
        (MahjongMeldKind::Kong(MahjongKongKind::Concealed), kind) => ([None; 3], Some(kind)),
        (MahjongMeldKind::Kong(MahjongKongKind::Melded), Some(kind)) => {
            ([Some(kind); 3], Some(Some(kind)))
        }
        _ => return,
    };
    let relative = presentation.relative;
    let own = relative == 0;
    let concealed = matches!(meld.kind, MahjongMeldKind::Kong(MahjongKongKind::Concealed));
    let size = match (own, concealed) {
        (true, true) => MahjongTileSize::OwnConcealedMeld,
        (true, false) => MahjongTileSize::OwnMeld,
        (false, true) => MahjongTileSize::MiniConcealedMeld,
        (false, false) => MahjongTileSize::Mini,
    };
    let (width, height, step, tile_width, tile_height) = if own {
        (
            MAHJONG_OWN_MELD_WIDTH,
            Layout::OWN_ROW_HEIGHT,
            Layout::OWN_STEP,
            Layout::OWN_WIDTH,
            Layout::OWN_HEIGHT,
        )
    } else {
        (
            MAHJONG_REMOTE_MELD_WIDTH,
            Layout::REMOTE_HEIGHT,
            Layout::MINI_STEP,
            Layout::MINI_WIDTH,
            Layout::MINI_HEIGHT,
        )
    };
    // The group and all four possible tile slots keep their identities through landing and animation cleanup.
    let group = spawn_node(
        commands,
        parent,
        Node {
            width: px(width),
            min_width: px(width),
            height: px(height),
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::Row,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    commands
        .entity(group)
        .insert(ZIndex(if matches!(relative, 1 | 2) {
            -(*index as i32)
        } else {
            *index as i32
        }));
    let landing = presentation.claim.and_then(|claim| match claim.claim {
        MahjongClaim::Chow { .. } => base
            .iter()
            .position(|kind| *kind == claim.tile.map(|tile| tile.kind())),
        MahjongClaim::Pung => Some(1),
        _ => None,
    });
    for (slot, kind) in base.into_iter().enumerate() {
        let tile = add_mahjong_tile_material(
            commands,
            group,
            MahjongTileVisual {
                elevation: 0.0,
                kind,
                size,
                index: *index,
                highlight: MahjongTileHighlight::None,
                deal: None,
                relative,
            },
            assets,
            materials,
        );
        stage_tile(commands, tile, presentation.claim, landing == Some(slot));
        *index += 1;
    }
    if let Some(kind) = stacked {
        let holder = spawn_node(
            commands,
            group,
            Node {
                position_type: PositionType::Absolute,
                left: px(step),
                top: px(height - tile_height),
                width: px(tile_width),
                height: px(tile_height),
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        let tile = add_mahjong_tile_material(
            commands,
            holder,
            MahjongTileVisual {
                elevation: MAHJONG_KONG_STACK_LIFT,
                kind,
                size,
                index: *index,
                highlight: MahjongTileHighlight::None,
                deal: None,
                relative,
            },
            assets,
            materials,
        );
        stage_tile(
            commands,
            tile,
            presentation.claim,
            presentation
                .claim
                .is_some_and(|claim| matches!(claim.claim, MahjongClaim::Kong)),
        );
        *index += 1;
    }
}

fn stage_tile(
    commands: &mut UiComposer,
    tile: Entity,
    claim: Option<&ActiveMahjongClaimPresentation>,
    landing: bool,
) {
    let Some(claim) = claim else { return };
    let (scale, offset) = mahjong_claim_held_tile_visual(claim.elapsed);
    commands.entity(tile).insert((
        MahjongClaimHeldTile {
            player: claim.player,
            landing,
        },
        if landing && claim.elapsed < mahjong_claim_landing_time() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
        if landing {
            UiTransform::IDENTITY
        } else {
            UiTransform {
                translation: Val2::px(0.0, offset),
                scale: Vec2::new(scale, 1.0),
                ..default()
            }
        },
    ));
    if landing {
        commands.entity(tile).insert(MahjongClaimLandingSlot {
            player: claim.player,
        });
    }
}
