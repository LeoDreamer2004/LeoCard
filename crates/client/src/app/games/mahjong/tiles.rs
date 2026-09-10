use super::{
    MahjongAssets, MahjongDealSpec, MahjongDealTile, MahjongHandTile, MahjongTileMaterial,
    MahjongTurnArrow, mahjong_local_light,
};
use crate::app::presentation::{PendingDealSound, ease_out_cubic};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use leocard_mahjong::MahjongTileKind;

const MAHJONG_DEAL_MOVE_DURATION: f32 = 0.28;

pub(super) fn mahjong_deal_spec(
    relative: u8,
    tile_index: usize,
    tile_count: usize,
    advance: f32,
) -> MahjongDealSpec {
    let final_x = (tile_index as f32 - (tile_count.saturating_sub(1)) as f32 * 0.5) * advance;
    let toward_wall = match relative {
        0 => -235.0,
        1 | 3 => 205.0,
        2 => -180.0,
        _ => -180.0,
    };
    MahjongDealSpec {
        start_offset: Vec2::new(-final_x, toward_wall),
        start_rotation: match relative {
            1 => -0.12,
            3 => 0.12,
            _ => ((tile_index * 19 % 5) as f32 - 2.0) * 0.025,
        },
    }
}

pub(super) fn queue_mahjong_deal_sound(commands: &mut Commands, assets: &UiAssets) {
    if assets.audio.deal_sounds.is_empty() {
        return;
    }
    commands.spawn(PendingDealSound {
        remaining: 0.0,
        variant: fastrand::usize(..assets.audio.deal_sounds.len()),
    });
}

#[expect(
    clippy::too_many_arguments,
    reason = "the tile builder keeps its material and animation inputs explicit"
)]
pub(super) fn add_mahjong_hand_tile(
    commands: &mut Commands,
    parent: Entity,
    kind: MahjongTileKind,
    action: Option<UiAction>,
    index: usize,
    deal: Option<MahjongDealSpec>,
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) -> Entity {
    let glyph = game_assets
        .tiles
        .get(&kind)
        .cloned()
        .expect("所有麻将牌面都应预加载");
    let height = game_assets
        .tile_heights
        .get(&kind)
        .cloned()
        .expect("所有麻将凹刻高度图都应预加载");
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(0.0, 0.0, if deal.is_some() { 0.0 } else { 1.0 }, -1.0),
        lighting: mahjong_local_light(0),
        glyph,
        height,
    });
    let base_rotation = 0.0;
    let entity = commands
        .spawn((
            MahjongHandTile {
                lift: 0.0,
                base_rotation,
                index: index as i32,
            },
            Node {
                width: px(50),
                height: px(68),
                min_width: px(50),
                margin: UiRect::right(px(-5)),
                overflow: Overflow::visible(),
                ..default()
            },
            MaterialNode(material),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(1), px(7), px(0), px(3)),
            ZIndex(index as i32),
            UiTransform::from_rotation(Rot2::radians(base_rotation)),
        ))
        .id();
    if let Some(action) = action {
        commands.entity(entity).insert((Button, action));
    }
    if let Some(deal) = deal {
        commands.entity(entity).insert(MahjongDealTile {
            elapsed: 0.0,
            start_offset: deal.start_offset,
            start_rotation: deal.start_rotation,
            final_offset: Vec2::ZERO,
            final_rotation: base_rotation,
            final_shadow_alpha: 0.0,
        });
    }
    commands.entity(parent).add_child(entity);
    entity
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query expresses the exact hand-tile material inputs"
)]
pub(super) fn sync_mahjong_hand_tile_materials(
    time: Res<Time>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut tiles: Query<
        (
            &mut MahjongHandTile,
            Option<&Interaction>,
            &MaterialNode<MahjongTileMaterial>,
            &mut UiTransform,
            &mut BoxShadow,
            &mut ZIndex,
        ),
        Without<MahjongDealTile>,
    >,
) {
    let smoothing = 1.0 - (-18.0 * time.delta_secs()).exp();
    for (mut tile, interaction, material_node, mut transform, mut shadow, mut z_index) in &mut tiles
    {
        let interaction = interaction.copied().unwrap_or(Interaction::None);
        let target = match interaction {
            Interaction::None => 0.0,
            Interaction::Hovered => 1.0,
            Interaction::Pressed => 0.62,
        };
        tile.lift += (target - tile.lift) * smoothing;
        transform.translation = Val2::px(0.0, -11.0 * tile.lift);
        transform.scale = Vec2::splat(1.0 + 0.025 * tile.lift);
        transform.rotation = Rot2::radians(tile.base_rotation * (1.0 - tile.lift));
        *z_index = ZIndex(if tile.lift > 0.05 {
            100 + tile.index
        } else {
            tile.index
        });
        *shadow = BoxShadow::new(
            Color::BLACK.with_alpha(tile.lift * 0.22),
            px(1),
            px(7.0 + tile.lift * 6.0),
            px(0),
            px(3.0 + tile.lift * 4.0),
        );
        let Some(mut material) = materials.get_mut(&material_node.0) else {
            continue;
        };
        material.params.x = match interaction {
            Interaction::None => 0.0,
            Interaction::Hovered => 1.0,
            Interaction::Pressed => 2.0,
        };
    }
}

pub(super) fn animate_mahjong_deal_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut tiles: Query<(
        Entity,
        &mut MahjongDealTile,
        &MaterialNode<MahjongTileMaterial>,
        &mut UiTransform,
        &mut BoxShadow,
    )>,
) {
    for (entity, mut deal, material_node, mut transform, mut shadow) in &mut tiles {
        deal.elapsed += time.delta_secs();
        let raw = (deal.elapsed / MAHJONG_DEAL_MOVE_DURATION).clamp(0.0, 1.0);
        let movement = ease_out_cubic(raw);
        let lift = (raw * std::f32::consts::PI).sin() * 12.0;
        let offset = deal.final_offset + deal.start_offset * (1.0 - movement);
        transform.translation = Val2::px(offset.x, offset.y - lift);
        transform.rotation =
            Rot2::radians(deal.final_rotation + deal.start_rotation * (1.0 - movement));
        transform.scale = Vec2::splat(0.82 + movement * 0.18);
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = (raw * 4.0).min(1.0);
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(deal.final_shadow_alpha * raw);
        }
        if raw >= 1.0 {
            commands.entity(entity).remove::<MahjongDealTile>();
        }
    }
}

pub(super) fn animate_mahjong_turn_arrows(
    time: Res<Time>,
    mut arrows: Query<(&MahjongTurnArrow, &mut ImageNode)>,
) {
    for (arrow, mut image) in &mut arrows {
        let phase = (time.elapsed_secs() * 2.15 - arrow.slot * 0.22).rem_euclid(1.35);
        let alpha = if phase < 0.62 {
            (phase / 0.62 * std::f32::consts::PI).sin().powf(0.72)
        } else {
            0.0
        };
        image.color = Color::WHITE.with_alpha(alpha * 0.92);
    }
}
