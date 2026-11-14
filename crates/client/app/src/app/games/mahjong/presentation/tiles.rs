use super::super::layout::MahjongLayout as Layout;
use super::super::{
    MahjongAssets, MahjongDealSpec, MahjongDealTile, MahjongDisplayedHand, MahjongMatchingTileKind,
    MahjongTileHighlight, MahjongTileMaterial, attach_mahjong_tile_surface, mahjong_local_light,
};
use crate::app::presentation::mahjong::MahjongTileGeometry;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mahjong::{MahjongMeldKind, MahjongTileKind, MahjongWinRecord, MahjongWind};
use leocard_protocol::MahjongPlayerView;
use leocard_ui::UiComposer;

pub(crate) const MAHJONG_KONG_STACK_LIFT: f32 = MahjongTileGeometry::THICKNESS * 30.0;

#[derive(Clone, Copy)]
pub(crate) enum MahjongTileSize {
    River,
    GuideHand,
    GuideMeld,
    GuideConcealedMeld,
    SettlementHand,
    Mini,
    Hint,
    MiniConcealedMeld,
    OwnMeld,
    OwnConcealedMeld,
    OwnHand,
    InkShowcase,
    MajorShowcase,
    HiddenSide,
    HiddenOpposite,
}

pub(crate) struct MahjongTileVisual {
    pub elevation: f32,
    pub kind: Option<MahjongTileKind>,
    pub size: MahjongTileSize,
    pub index: usize,
    pub highlight: MahjongTileHighlight,
    pub deal: Option<MahjongDealSpec>,
    pub relative: u8,
}

impl MahjongTileSize {
    pub(in super::super) fn dimensions(self) -> (f32, f32, f32) {
        match self {
            MahjongTileSize::River => (
                Layout::RIVER_WIDTH,
                Layout::RIVER_HEIGHT,
                Layout::RIVER_STEP - Layout::RIVER_WIDTH,
            ),
            MahjongTileSize::GuideHand
            | MahjongTileSize::GuideMeld
            | MahjongTileSize::GuideConcealedMeld => (33.0, 45.0, -3.0),
            MahjongTileSize::SettlementHand => (46.0, 63.0, -4.0),
            MahjongTileSize::Mini | MahjongTileSize::MiniConcealedMeld => (
                Layout::MINI_WIDTH,
                Layout::MINI_HEIGHT,
                Layout::MINI_STEP - Layout::MINI_WIDTH,
            ),
            MahjongTileSize::Hint => {
                if cfg!(target_os = "android") {
                    (36.0, 50.0, -4.0)
                } else {
                    (27.0, 37.0, -3.0)
                }
            }
            MahjongTileSize::InkShowcase => (54.0, 73.0, -7.0),
            MahjongTileSize::MajorShowcase => (64.0, 86.0, -7.0),
            MahjongTileSize::OwnMeld
            | MahjongTileSize::OwnConcealedMeld
            | MahjongTileSize::OwnHand => (
                Layout::OWN_WIDTH,
                Layout::OWN_HEIGHT,
                Layout::OWN_STEP - Layout::OWN_WIDTH,
            ),
            MahjongTileSize::HiddenSide => (16.0, 10.4, 0.3),
            MahjongTileSize::HiddenOpposite => (16.0, 8.96, 0.1),
        }
    }
}

pub(crate) fn add_mahjong_tile_material(
    commands: &mut UiComposer,
    parent: Entity,
    visual: MahjongTileVisual,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> Entity {
    let MahjongTileVisual {
        elevation,
        kind,
        size,
        index,
        highlight,
        deal,
        relative,
    } = visual;
    let (width, height, overlap) = size.dimensions();
    let back = kind.is_none();
    let glyph = kind.map_or_else(
        || game_assets.visuals.tile_back.clone(),
        |kind| {
            game_assets
                .visuals
                .tiles
                .get(&kind)
                .cloned()
                .expect("所有麻将牌面都应预加载")
        },
    );
    let height_texture = kind.map_or_else(
        || game_assets.visuals.tile_back.clone(),
        |kind| {
            game_assets
                .visuals
                .tile_heights
                .get(&kind)
                .cloned()
                .expect("所有麻将凹刻高度图都应预加载")
        },
    );
    let node = Node {
        width: px(width),
        height: px(height),
        min_width: px(width),
        margin: UiRect::right(px(overlap)),
        border_radius: BorderRadius::all(px(3)),
        overflow: Overflow::visible(),
        ..default()
    };
    let final_rotation = 0.0;
    let final_offset = if matches!(size, MahjongTileSize::OwnHand) {
        Vec2::new(0.0, -Layout::OWN_HAND_LIFT)
    } else {
        Vec2::ZERO
    };
    let entity = commands.child(
        parent,
        (
            node,
            Pickable::IGNORE,
            // Farther rows must be painted first, so their shadows cannot cover the nearer faces.
            ZIndex(if matches!(size, MahjongTileSize::River) {
                Layout::river_tile_layer(relative, index)
            } else if matches!(relative, 1 | 2) {
                -(index as i32)
            } else {
                index as i32
            }),
            UiTransform {
                translation: Val2::px(final_offset.x, final_offset.y),
                rotation: Rot2::radians(final_rotation),
                ..default()
            },
        ),
    );

    let table_projection = matches!(
        size,
        MahjongTileSize::River
            | MahjongTileSize::Mini
            | MahjongTileSize::MiniConcealedMeld
            | MahjongTileSize::HiddenSide
            | MahjongTileSize::HiddenOpposite
            | MahjongTileSize::OwnMeld
            | MahjongTileSize::OwnConcealedMeld
    );
    let material = commands.material(
        entity,
        materials,
        MahjongTileMaterial {
            table_projection,
            overlay_projection: matches!(
                size,
                MahjongTileSize::OwnMeld | MahjongTileSize::OwnConcealedMeld
            ),
            params: Vec4::new(
                0.0,
                if back { 1.0 } else { 0.0 },
                if deal.is_some() { 0.0 } else { 1.0 },
                match size {
                    MahjongTileSize::HiddenSide => -2.0,
                    MahjongTileSize::HiddenOpposite => -3.0,
                    MahjongTileSize::GuideHand | MahjongTileSize::SettlementHand => -1.0,
                    MahjongTileSize::OwnMeld
                    | MahjongTileSize::InkShowcase
                    | MahjongTileSize::MajorShowcase => -4.0,
                    MahjongTileSize::OwnHand => -6.0,
                    MahjongTileSize::GuideMeld => -4.0,
                    MahjongTileSize::OwnConcealedMeld
                    | MahjongTileSize::MiniConcealedMeld
                    | MahjongTileSize::GuideConcealedMeld => -5.0,
                    MahjongTileSize::River | MahjongTileSize::Mini | MahjongTileSize::Hint => 0.0,
                },
            ),
            lighting: mahjong_local_light(relative),
            highlight: highlight.overlay(),
            elevation,
            glyph,
            height: height_texture,
            ..default()
        },
    );
    attach_mahjong_tile_surface(commands, entity, material, table_projection);
    if let Some(kind) = kind
        && !matches!(
            size,
            MahjongTileSize::GuideHand
                | MahjongTileSize::SettlementHand
                | MahjongTileSize::GuideMeld
                | MahjongTileSize::GuideConcealedMeld
                | MahjongTileSize::InkShowcase
                | MahjongTileSize::MajorShowcase
                | MahjongTileSize::Hint
                | MahjongTileSize::OwnHand
        )
    {
        commands
            .entity(entity)
            .insert(MahjongMatchingTileKind(kind));
    }
    if let Some(deal) = deal {
        commands.entity(entity).init(MahjongDealTile {
            elapsed: 0.0,
            falling: deal.falling,
            start_offset: deal.start_offset,
            start_rotation: deal.start_rotation,
            final_offset,
            final_rotation,
        });
    }
    entity
}

pub(crate) struct MahjongWinTileSizes {
    pub meld: MahjongTileSize,
    pub hand: MahjongTileSize,
}

pub(crate) struct MahjongWinTileRow {
    pub tiles: Vec<Entity>,
    pub width: f32,
}

pub(crate) fn render_mahjong_win_tile_row(
    commands: &mut UiComposer,
    row: Entity,
    player: &MahjongPlayerView,
    winner: &MahjongWinRecord,
    sizes: MahjongWinTileSizes,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> MahjongWinTileRow {
    let mut kinds = Vec::new();
    for meld in &player.melds {
        let meld_tiles = match (meld.kind, meld.tile) {
            (MahjongMeldKind::Chow, Some(MahjongTileKind::Suited { suit, rank })) => (0..3)
                .map(|offset| Some(MahjongTileKind::suited(suit, rank + offset)))
                .collect::<Vec<_>>(),
            (MahjongMeldKind::Pung, kind) => vec![kind; 3],
            (MahjongMeldKind::Kong(_), kind) => vec![kind; 4],
            _ => continue,
        };
        for kind in meld_tiles {
            kinds.push((kind, sizes.meld));
        }
    }
    let displayed = MahjongDisplayedHand::new(
        player.revealed_hand.as_deref().unwrap_or_default(),
        Some(winner),
    );
    for tile in displayed.regular {
        kinds.push((Some(tile.kind()), sizes.hand));
    }
    kinds.push((Some(winner.winning_tile.kind()), sizes.hand));
    let mut tiles = Vec::with_capacity(kinds.len());
    let mut width = 0.0;
    for (index, (kind, size)) in kinds.into_iter().enumerate() {
        let (tile_width, _, overlap) = size.dimensions();
        width += tile_width + overlap;
        let entity = add_mahjong_tile_material(
            commands,
            row,
            MahjongTileVisual {
                elevation: 0.0,
                kind,
                size,
                index,
                highlight: MahjongTileHighlight::None,
                deal: None,
                relative: 0,
            },
            assets,
            materials,
        );
        tiles.push(entity);
    }
    MahjongWinTileRow { tiles, width }
}

pub(crate) fn wind_label(wind: MahjongWind) -> &'static str {
    match wind {
        MahjongWind::East => "東",
        MahjongWind::South => "南",
        MahjongWind::West => "西",
        MahjongWind::North => "北",
    }
}
