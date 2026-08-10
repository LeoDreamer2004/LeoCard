//! 德州扑克摊牌到结算弹窗之间的最佳五张牌演出。

use super::*;

#[derive(Component)]
pub(in crate::app) struct TexasShowdownRevealRoot;

#[derive(Component)]
pub(in crate::app) struct TexasShowdownBackdrop;

#[derive(Component)]
pub(in crate::app) struct TexasShowdownBestCard {
    pub(in crate::app) source: Vec2,
    pub(in crate::app) target: Vec2,
    pub(in crate::app) delay: f32,
    pub(in crate::app) start_scale: f32,
}

#[derive(Component)]
pub(in crate::app) struct TexasShowdownTitle;

#[derive(Component)]
pub(in crate::app) struct TexasShowdownTitleText;

#[derive(Component)]
pub(in crate::app) struct TexasShowdownUnderline;

pub(in crate::app) fn animate_texas_showdown_reveal(
    animation: Res<GameSummaryAnimation>,
    mut roots: Query<&mut Visibility, With<TexasShowdownRevealRoot>>,
    mut visuals: ParamSet<(
        Query<&mut BackgroundColor, With<TexasShowdownBackdrop>>,
        Query<(
            &TexasShowdownBestCard,
            &mut UiTransform,
            &mut ImageNode,
            &mut BorderColor,
        )>,
        Query<&mut UiTransform, With<TexasShowdownTitle>>,
        Query<(&mut UiTransform, &mut BackgroundColor), With<TexasShowdownUnderline>>,
    )>,
    mut title_texts: Query<&mut TextColor, With<TexasShowdownTitleText>>,
) {
    let elapsed = animation.elapsed + TEXAS_SHOWDOWN_REVEAL_DURATION;
    let active = animation.texas_hand_number.is_some() && animation.elapsed < 0.0 && elapsed >= 0.0;
    for mut visibility in &mut roots {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !active {
        return;
    }

    let backdrop_entry = ease_out_cubic((elapsed / 0.34).clamp(0.0, 1.0));
    let exit = ((elapsed - 2.34) / 0.26).clamp(0.0, 1.0);
    for mut background in &mut visuals.p0() {
        background.0 = Color::BLACK.with_alpha(backdrop_entry * (1.0 - exit) * 0.48);
    }
    for (card, mut transform, mut image, mut border) in &mut visuals.p1() {
        let progress = ((elapsed - card.delay) / 0.52).clamp(0.0, 1.0);
        let movement = ease_out_cubic(progress);
        let mut position = card.source.lerp(card.target, movement);
        position.y -= (progress * std::f32::consts::PI).sin() * 28.0;
        transform.translation = Val2::px(position.x - card.target.x, position.y - card.target.y);
        let settle = ((elapsed - 1.42) / 0.48).clamp(0.0, 1.0);
        let pulse = (settle * std::f32::consts::PI).sin() * 0.045;
        transform.scale =
            Vec2::splat(card.start_scale + (1.0 - card.start_scale) * movement + pulse);
        let alpha = ((elapsed - card.delay) / 0.10).clamp(0.0, 1.0) * (1.0 - exit);
        image.color = Color::WHITE.with_alpha(alpha);
        border.set_all(Color::srgb(0.90, 0.73, 0.28).with_alpha(alpha * (0.48 + settle * 0.42)));
    }

    let title_progress = ease_out_cubic(((elapsed - 1.30) / 0.34).clamp(0.0, 1.0));
    for mut transform in &mut visuals.p2() {
        transform.translation = Val2::px(0.0, 16.0 * (1.0 - title_progress) - exit * 8.0);
        transform.scale = Vec2::splat(0.92 + title_progress * 0.08);
    }
    for mut color in &mut title_texts {
        color.0 = Color::srgb(0.96, 0.78, 0.33).with_alpha(title_progress * (1.0 - exit));
    }
    let line_progress = ease_out_cubic(((elapsed - 1.46) / 0.42).clamp(0.0, 1.0));
    for (mut transform, mut background) in &mut visuals.p3() {
        transform.scale.x = line_progress;
        background.0 =
            Color::srgb(0.90, 0.72, 0.27).with_alpha(line_progress * (1.0 - exit) * 0.78);
    }
}
