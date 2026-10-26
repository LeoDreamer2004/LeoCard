use super::super::GuessScorePlayback;
use super::state::GuessFeedback;
use crate::app::presentation::{ACCENT, READY, ease_out_cubic};
use bevy::prelude::*;
use leocard_ui::{TransitionOpacity, TransitionVisuals, UiComposer, fade_panel};

#[derive(Component)]
pub(in super::super) struct GuessCellReveal {
    pub row: usize,
    pub column: usize,
    pub color: Color,
    pub content: Entity,
    applied: (f32, f32),
}

impl GuessCellReveal {
    pub(in super::super) fn new(row: usize, column: usize, color: Color, content: Entity) -> Self {
        Self {
            row,
            column,
            color,
            content,
            applied: (-1.0, -1.0),
        }
    }
}

#[derive(Component)]
pub(in super::super) enum RevealPanel {
    Placeholder,
    Heading,
    Banner,
}

#[derive(Component)]
pub(in super::super) struct GuessReadyControls;

#[derive(Component)]
pub(in super::super) struct GuessWinner;

#[derive(Component)]
pub(in super::super) struct RevealPixel {
    pub velocity: Vec2,
    pub delay: f32,
}

pub(in super::super) fn animate_cells(
    mut commands: UiComposer,
    playback: Res<GuessFeedback>,
    children: Query<&Children>,
    mut cells: Query<(
        Entity,
        &mut GuessCellReveal,
        &TransitionOpacity,
        &mut ImageNode,
        &mut BorderColor,
    )>,
    mut visuals: TransitionVisuals<Without<GuessCellReveal>>,
    mut transforms: Query<&mut UiTransform>,
) {
    for (entity, mut cell, opacity, mut image, mut border) in &mut cells {
        let progress = ease_out_cubic(playback.cell_progress(cell.row, cell.column));
        let opacity = opacity.factor();
        if cell.applied == (progress, opacity) {
            continue;
        }
        cell.applied = (progress, opacity);
        image.color = Color::srgb(0.65, 0.65, 0.65)
            .mix(&cell.color, progress)
            .with_alpha(opacity);
        if let Some(base) = commands.declared::<BorderColor>(entity) {
            border.top = base.top.with_alpha(base.top.alpha() * opacity);
            border.right = base.right.with_alpha(base.right.alpha() * opacity);
            border.bottom = base.bottom.with_alpha(base.bottom.alpha() * opacity);
            border.left = base.left.with_alpha(base.left.alpha() * opacity);
        }
        fade_panel(
            cell.content,
            progress * opacity,
            &children,
            &mut visuals,
            &mut commands,
        );
        if let Ok(mut transform) = transforms.get_mut(cell.content) {
            transform.translation = Val2::px(0.0, 3.0 * (1.0 - progress));
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "independent presentation components share the reveal clock"
)]
pub(in super::super) fn animate_reveal(
    mut commands: UiComposer,
    playback: Res<GuessFeedback>,
    scores: Res<GuessScorePlayback>,
    panels: Query<(Entity, &RevealPanel)>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals<Without<RevealPixel>>,
    mut transforms: Query<&mut UiTransform>,
    mut ready: Query<&mut Visibility, With<GuessReadyControls>>,
    mut winners: Query<&mut Outline, With<GuessWinner>>,
    mut pixels: Query<(Entity, &RevealPixel, &mut BackgroundColor)>,
) {
    let progress = ease_out_cubic(playback.answer_progress());
    let banner = playback.banner_opacity();
    for (entity, panel) in &panels {
        let opacity = match panel {
            RevealPanel::Placeholder => 1.0 - progress,
            RevealPanel::Heading => progress,
            RevealPanel::Banner => banner,
        };
        fade_panel(entity, opacity, &children, &mut visuals, &mut commands);
        if let Ok(mut transform) = transforms.get_mut(entity) {
            let offset = match panel {
                RevealPanel::Placeholder => -5.0 * progress,
                RevealPanel::Heading => 6.0 * (1.0 - progress),
                RevealPanel::Banner => 12.0 * (1.0 - progress),
            };
            transform.translation.y = px(offset);
            if matches!(panel, RevealPanel::Banner) {
                transform.scale = Vec2::splat(0.94 + 0.06 * progress);
            }
        }
    }
    for mut visibility in &mut ready {
        *visibility = if playback.busy() || scores.busy() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let elapsed = playback.answer_elapsed().unwrap_or(-1.0);
    let glow = if (0.0..1.25).contains(&elapsed) {
        (elapsed / 0.15).min(1.0) * ((1.25 - elapsed) / 0.45).min(1.0)
    } else {
        0.0
    };
    for mut outline in &mut winners {
        outline.width = px(3.0 + 3.0 * glow);
        outline.color = READY.mix(&ACCENT, glow);
    }
    for (entity, pixel, mut background) in &mut pixels {
        let local = elapsed - pixel.delay;
        if let Ok(mut transform) = transforms.get_mut(entity) {
            transform.translation = Val2::px(
                pixel.velocity.x * local.max(0.0),
                pixel.velocity.y * local.max(0.0) + 36.0 * local.max(0.0).powi(2),
            );
        }
        background.0 = ACCENT
            .with_alpha((local / 0.08).clamp(0.0, 1.0) * ((0.85 - local) / 0.30).clamp(0.0, 1.0));
    }
}
