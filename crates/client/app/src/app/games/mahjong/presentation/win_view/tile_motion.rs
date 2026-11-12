use super::major::MajorShowcaseSequence;
use super::timeline::{WinShowcaseTiming, smooth};
use crate::app::games::mahjong::{MahjongTileMaterial, MahjongTileSurface};
use crate::app::presentation::{GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;
use std::f32::consts;

#[derive(Clone, Copy)]
pub(super) enum ShowcaseTileStyle {
    Ink,
    Major,
}

#[derive(Component)]
pub(crate) struct MahjongShowcaseTile {
    timing: WinShowcaseTiming,
    style: ShowcaseTileStyle,
    index: usize,
    count: usize,
}

impl MahjongShowcaseTile {
    pub(super) fn new(
        timing: WinShowcaseTiming,
        style: ShowcaseTileStyle,
        index: usize,
        count: usize,
    ) -> Self {
        Self {
            timing,
            style,
            index,
            count,
        }
    }
}

pub(crate) fn animate_mahjong_showcase_tiles(
    animation: Res<GameSummaryAnimation>,
    mut tiles: Query<(
        &MahjongShowcaseTile,
        &MahjongTileSurface,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
) {
    for (tile, surface, mut transform, mut visibility) in &mut tiles {
        let local = tile.timing.local(animation.elapsed);
        let winning = tile.index + 1 == tile.count;
        let (start, winning_start, duration, exit, fade_time, curve) = match tile.style {
            ShowcaseTileStyle::Ink => (0.47, 0.93, 0.40, 2.43, 0.40, 0.0),
            ShowcaseTileStyle::Major => (
                MajorShowcaseSequence::TILE_START,
                MajorShowcaseSequence::WINNING_TILE_START,
                MajorShowcaseSequence::TILE_FLIGHT,
                tile.timing.duration - MajorShowcaseSequence::FADE,
                0.60,
                12.0,
            ),
        };
        let contact_duration = match tile.style {
            ShowcaseTileStyle::Ink => 0.18,
            ShowcaseTileStyle::Major => MajorShowcaseSequence::TILE_SETTLE,
        };
        let age = local
            - if winning {
                winning_start
            } else {
                start + tile.index as f32 * 0.014
            };
        let active = tile.timing.active(local) && age > 0.0;
        *visibility = if active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if !active {
            continue;
        }
        let progress = ease_out_cubic((age / duration).clamp(0.0, 1.0));
        let elevation = if winning { 56.0 } else { 29.0 } * (1.0 - progress);
        let landed = age - duration;
        let contact = if (0.0..contact_duration).contains(&landed) {
            (landed / contact_duration * consts::PI).sin() * if winning { 1.7 } else { 0.7 }
        } else {
            0.0
        };
        let axis = tile.index as f32 / tile.count.saturating_sub(1).max(1) as f32 * 2.0 - 1.0;
        let rest_y = -curve * (1.0 - axis * axis);
        let rest_rotation = axis * curve / 3.0;
        transform.translation = Val2::px(
            -if winning { 7.0 } else { 22.0 } * (1.0 - progress),
            rest_y - elevation + contact + smooth((local - exit) / fade_time) * 8.0,
        );
        transform.rotation =
            Rot2::degrees(rest_rotation + if winning { 2.5 } else { -1.7 } * (1.0 - progress));
        transform.scale = Vec2::splat(1.0 + if winning { 0.04 } else { 0.015 } * (1.0 - progress));
        if let Some(mut material) = materials.get_mut(&surface.0) {
            material.params.z = smooth(age / 0.13) * (1.0 - smooth((local - exit) / fade_time));
            material.elevation = elevation * 0.12;
        }
    }
}
