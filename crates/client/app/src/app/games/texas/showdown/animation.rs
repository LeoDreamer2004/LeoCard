use super::state::*;
use super::timeline::TexasShowdownRound;
use crate::app::presentation::{GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;
use std::f32::consts;

#[expect(
    clippy::type_complexity,
    reason = "the ParamSet keeps overlapping Bevy showdown queries disjoint"
)]
pub(in crate::app::games::texas) fn animate_texas_showdown_reveal(
    animation: Res<GameSummaryAnimation>,
    mut roots: Query<(&TexasShowdownRound, &mut Visibility), With<TexasShowdownRevealRoot>>,
    mut visuals: ParamSet<(
        Query<(&TexasShowdownRound, &mut BackgroundColor), With<TexasShowdownBackdrop>>,
        Query<(
            &TexasShowdownRound,
            &TexasShowdownBestCard,
            &mut UiTransform,
            &mut ImageNode,
            &mut BorderColor,
        )>,
        Query<(&TexasShowdownRound, &mut UiTransform), With<TexasShowdownTitle>>,
        Query<
            (&TexasShowdownRound, &mut UiTransform, &mut BackgroundColor),
            With<TexasShowdownUnderline>,
        >,
    )>,
    mut title_texts: Query<(&TexasShowdownRound, &mut TextColor), With<TexasShowdownTitleText>>,
) {
    for (round, mut visibility) in &mut roots {
        *visibility = if round.active(&animation) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (round, mut background) in &mut visuals.p0() {
        let elapsed = round.elapsed(&animation);
        let backdrop_entry = ease_out_cubic((elapsed / 0.34).clamp(0.0, 1.0));
        let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
        background.0 = Color::BLACK.with_alpha(backdrop_entry * (1.0 - exit) * 0.48);
    }
    for (round, card, mut transform, mut image, mut border) in &mut visuals.p1() {
        let elapsed = round.elapsed(&animation);
        let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
        let progress = ((elapsed - card.delay) / 0.52).clamp(0.0, 1.0);
        let movement = ease_out_cubic(progress);
        let mut position = card.source.lerp(card.target, movement);
        position.y -= (progress * consts::PI).sin() * 28.0;
        transform.translation = Val2::px(position.x - card.target.x, position.y - card.target.y);
        let settle = ((elapsed - 1.42) / 0.48).clamp(0.0, 1.0);
        let pulse = (settle * consts::PI).sin() * 0.045;
        transform.scale = card.start_scale.lerp(Vec2::ONE, movement) + Vec2::splat(pulse);
        let alpha = ((elapsed - card.delay) / 0.10).clamp(0.0, 1.0) * (1.0 - exit);
        image.color = Color::WHITE.with_alpha(alpha);
        border.set_all(Color::srgb(0.90, 0.73, 0.28).with_alpha(alpha * (0.48 + settle * 0.42)));
    }

    for (round, mut transform) in &mut visuals.p2() {
        let elapsed = round.elapsed(&animation);
        let title_progress = ease_out_cubic(((elapsed - 1.30) / 0.34).clamp(0.0, 1.0));
        let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
        transform.translation = Val2::px(0.0, 16.0 * (1.0 - title_progress) - exit * 8.0);
        transform.scale = Vec2::splat(0.92 + title_progress * 0.08);
    }
    for (round, mut color) in &mut title_texts {
        let elapsed = round.elapsed(&animation);
        let title_progress = ease_out_cubic(((elapsed - 1.30) / 0.34).clamp(0.0, 1.0));
        let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
        color.0 = Color::srgb(0.96, 0.78, 0.33).with_alpha(title_progress * (1.0 - exit));
    }
    for (round, mut transform, mut background) in &mut visuals.p3() {
        let elapsed = round.elapsed(&animation);
        let line_progress = ease_out_cubic(((elapsed - 1.46) / 0.42).clamp(0.0, 1.0));
        let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
        transform.scale.x = line_progress;
        background.0 =
            Color::srgb(0.90, 0.72, 0.27).with_alpha(line_progress * (1.0 - exit) * 0.78);
    }
}
