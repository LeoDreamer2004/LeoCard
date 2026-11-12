use super::super::{
    MahjongCalligraphy, MahjongTileHighlight, MahjongTileMaterial, MahjongTileRestRotation,
    MahjongTileSurface, MahjongWinEffect, MahjongWinEffectTier, MahjongWinningHand,
    mahjong_win_effect_tier, mahjong_win_reveal_duration, mahjong_win_stage_start,
};
use super::MAHJONG_WIN_PUSH_DURATION;
use super::win_view::mahjong_win_effect_visual;
use crate::app::presentation::{GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;
use leocard_mahjong::MahjongWinRecord;
use leocard_protocol::MahjongHandResultView;
use leocard_ui::UiComposer;

#[derive(Clone, Copy)]
pub(crate) struct MahjongWinTileCue {
    tier: MahjongWinEffectTier,
    start: f32,
    duration: f32,
}

#[derive(Component)]
pub(crate) struct MahjongWinTileShake {
    reveal_duration: f32,
    cues: Vec<MahjongWinTileCue>,
    /// 手牌进入胡牌展示时恢复原色，牌河中的和牌张保持红色。
    hand_reveal_start: Option<f32>,
}

pub(crate) fn mahjong_win_tile_cues(
    result: &MahjongHandResultView,
    matches: impl Fn(&MahjongWinRecord) -> bool,
) -> Vec<MahjongWinTileCue> {
    result
        .winners
        .iter()
        .enumerate()
        .filter(|(_, winner)| matches(winner))
        .map(|(index, winner)| {
            let tier = mahjong_win_effect_tier(winner);
            MahjongWinTileCue {
                tier,
                start: mahjong_win_stage_start(result, index),
                duration: match tier {
                    MahjongWinEffectTier::Normal => 0.90,
                    MahjongWinEffectTier::HighTotal => 0.68,
                    MahjongWinEffectTier::MajorFan => 0.84,
                },
            }
        })
        .collect()
}

pub(crate) fn mark_mahjong_win_tile(
    commands: &mut UiComposer,
    entity: Entity,
    result: &MahjongHandResultView,
    cues: Vec<MahjongWinTileCue>,
    hand_reveal_start: Option<f32>,
) {
    if cues.is_empty() {
        return;
    }
    commands.entity(entity).insert(MahjongWinTileShake {
        reveal_duration: mahjong_win_reveal_duration(result),
        cues,
        hand_reveal_start,
    });
}

#[expect(
    clippy::type_complexity,
    reason = "the animation preserves each tile's local pose and only temporarily promotes its UI layer"
)]
pub(crate) fn animate_mahjong_win_tile_shakes(
    mut commands: Commands,
    animation: Res<GameSummaryAnimation>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut tiles: Query<(
        Entity,
        &MahjongWinTileShake,
        &MahjongTileSurface,
        &mut UiTransform,
        Option<&mut GlobalZIndex>,
        Option<&MahjongTileRestRotation>,
    )>,
) {
    for (entity, shake, material_node, mut transform, z_index, rest) in &mut tiles {
        let rest_rotation = rest.map_or(0.0, |rest| rest.0);
        let time = animation.elapsed + shake.reveal_duration;
        let highlight = if shake.hand_reveal_start.is_some_and(|start| time >= start) {
            MahjongTileHighlight::None
        } else {
            MahjongTileHighlight::Red
        };
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.highlight = highlight.overlay();
        }
        let active = shake.cues.iter().find_map(|cue| {
            (cue.start..cue.start + cue.duration)
                .contains(&time)
                .then_some((*cue, time - cue.start))
        });
        if let Some((cue, local)) = active {
            let (rotation, scale) = mahjong_win_tile_pose(cue, local);
            transform.translation = Val2::ZERO;
            transform.rotation = Rot2::radians(rest_rotation + rotation);
            transform.scale = Vec2::splat(scale);
            if let Some(mut z_index) = z_index {
                *z_index = GlobalZIndex(1010);
            } else {
                commands.entity(entity).insert(GlobalZIndex(1010));
            }
        } else {
            *transform = UiTransform::from_rotation(Rot2::radians(rest_rotation));
            if z_index.is_some() {
                commands.entity(entity).remove::<GlobalZIndex>();
            }
        }
    }
}

fn mahjong_win_tile_pose(cue: MahjongWinTileCue, local: f32) -> (f32, f32) {
    let envelope = (local / 0.08).clamp(0.0, 1.0) * ((cue.duration - local) / 0.16).clamp(0.0, 1.0);
    let amplitude = match cue.tier {
        MahjongWinEffectTier::Normal => 0.045,
        MahjongWinEffectTier::HighTotal => 0.085,
        MahjongWinEffectTier::MajorFan => 0.16,
    };
    let rotation = ((local * 42.0).sin() + (local * 71.0).sin() * 0.28) * amplitude * envelope;
    let scale = 1.0 + (local * 27.0).sin().abs() * 0.025 * envelope;
    (rotation, scale)
}

pub(crate) fn animate_mahjong_win_effects(
    animation: Res<GameSummaryAnimation>,
    mut effects: Query<(
        &MahjongWinEffect,
        &MahjongCalligraphy,
        &mut UiTransform,
        &mut Visibility,
    )>,
    mut images: Query<&mut ImageNode>,
) {
    for (effect, calligraphy, mut transform, mut visibility) in &mut effects {
        let Some((scale, alpha, offset_y)) =
            mahjong_win_effect_visual(animation.elapsed, effect.reveal_duration, effect.tier)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        calligraphy.set_alpha(alpha, &mut images);
        transform.translation = Val2::px(0.0, offset_y);
        transform.scale = Vec2::splat(scale);
        *visibility = Visibility::Visible;
    }
}

pub(crate) fn mahjong_winning_hand_progress(
    summary_elapsed: f32,
    reveal_duration: f32,
    start: f32,
) -> f32 {
    ((summary_elapsed + reveal_duration - start) / MAHJONG_WIN_PUSH_DURATION).clamp(0.0, 1.0)
}

pub(crate) fn apply_mahjong_winning_hand_visual(
    transform: &mut UiTransform,
    relative: u8,
    base_rotation: f32,
    progress: f32,
) {
    let progress = ease_out_cubic(progress);
    if relative == 0 {
        transform.translation = Val2::px(0.0, 8.0 * (1.0 - progress));
        transform.rotation = Rot2::radians(base_rotation);
        transform.scale = Vec2::splat(0.97 + progress * 0.03);
        return;
    }
    let distance = 12.0 * (1.0 - progress);
    let offset = match relative {
        0 => Vec2::new(0.0, distance),
        1 => Vec2::new(distance, 0.0),
        2 => Vec2::new(0.0, -distance),
        _ => Vec2::new(-distance, 0.0),
    };
    transform.translation = Val2::px(offset.x, offset.y);
    transform.rotation = Rot2::radians(base_rotation);
    transform.scale = Vec2::new(0.96 + progress * 0.04, 0.02 + progress * 0.98);
}

pub(crate) fn animate_mahjong_winning_hands(
    animation: Res<GameSummaryAnimation>,
    mut hands: Query<(&MahjongWinningHand, &mut UiTransform, &mut Visibility)>,
) {
    for (hand, mut transform, mut visibility) in &mut hands {
        let progress =
            mahjong_winning_hand_progress(animation.elapsed, hand.reveal_duration, hand.start);
        *visibility = if hand.relative == 0 || progress > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        apply_mahjong_winning_hand_visual(
            &mut transform,
            hand.relative,
            hand.base_rotation,
            progress,
        );
    }
}
