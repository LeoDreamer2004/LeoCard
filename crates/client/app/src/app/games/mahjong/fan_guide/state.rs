use super::super::{MahjongAssets, MahjongTileMaterial, MahjongTileSurface, MahjongUiState};
use super::view::render_fan_guide;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::UiComposer;

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideScroll;

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideRoot(pub(super) u16);

#[derive(Component)]
pub(in super::super) struct MahjongFanGuideTile;

pub(in super::super) fn sync_mahjong_fan_guide(
    mut commands: UiComposer,
    ui: Res<MahjongUiState>,
    roots: Query<(Entity, &MahjongFanGuideRoot)>,
    assets: Res<UiAssets>,
    game_assets: Res<MahjongAssets>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let tier = (ui.fan_guide_open || ui.fan_guide_progress > 0.0).then_some(ui.fan_guide_tier);
    let mut current = None;
    for (entity, root) in &roots {
        if Some(root.0) == tier {
            current = Some(entity);
        } else {
            commands.entity(entity).despawn();
        }
    }
    let Some(tier) = tier else {
        return;
    };
    if current.is_some() {
        return;
    }
    let root = commands
        .spawn((
            MahjongFanGuideRoot(tier),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            GlobalZIndex(2200),
            Pickable::IGNORE,
        ))
        .id();
    render_fan_guide(
        &mut commands,
        root,
        tier,
        ui.fan_guide_progress,
        &assets,
        &game_assets,
        &mut materials,
    );
}

pub(in super::super) fn animate_mahjong_fan_guide_tiles(
    ui: Res<MahjongUiState>,
    mut tiles: Query<(&MahjongTileSurface, Option<&mut BoxShadow>), With<MahjongFanGuideTile>>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    let progress = ui.fan_guide_progress;
    let opacity = progress * progress * (3.0 - 2.0 * progress);
    for (node, shadow) in &mut tiles {
        if let Some(mut material) = materials.get_mut(&node.0) {
            material.params.z = opacity;
        }
        if let Some(mut shadow) = shadow {
            *shadow = BoxShadow::new(
                Color::BLACK.with_alpha(0.24 * opacity),
                px(2),
                px(5),
                px(0),
                px(4),
            );
        }
    }
}
