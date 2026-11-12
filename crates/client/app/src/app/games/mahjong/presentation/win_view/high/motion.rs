use super::super::timeline::smooth;
use super::MahjongWinInkMaterial;
use crate::app::presentation::{GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;

pub(crate) const MAHJONG_INK_SHOWCASE_DURATION: f32 = 3.15;

#[derive(Component)]
pub(crate) struct MahjongWinShowcase {
    pub(super) reveal_duration: f32,
    pub(super) start: f32,
    pub(super) backdrop: Entity,
    pub(super) ink: Entity,
    pub(super) ink_material: Handle<MahjongWinInkMaterial>,
    pub(super) title: Entity,
    pub(super) name: Entity,
}

pub(crate) fn animate_mahjong_win_showcases(
    animation: Res<GameSummaryAnimation>,
    showcases: Query<(Entity, &MahjongWinShowcase)>,
    mut parts: Query<(&mut UiTransform, &mut Visibility)>,
    mut images: Query<&mut ImageNode>,
    mut names: Query<&mut TextColor>,
    mut backdrops: Query<&mut BackgroundColor>,
    mut inks: ResMut<Assets<MahjongWinInkMaterial>>,
) {
    for (entity, showcase) in &showcases {
        let local = animation.elapsed + showcase.reveal_duration - showcase.start;
        let active = (0.0..MAHJONG_INK_SHOWCASE_DURATION).contains(&local);
        if let Ok((_, mut visibility)) = parts.get_mut(entity) {
            *visibility = if active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if !active {
            continue;
        }
        let entry = smooth(local / 0.4);
        let fade = 1.0 - smooth((local - 2.46) / 0.45);
        if let Ok(mut backdrop) = backdrops.get_mut(showcase.backdrop) {
            backdrop.0 =
                Color::BLACK.with_alpha(0.26 * entry * (1.0 - smooth((local - 2.65) / 0.5)));
        }
        if let Some(mut ink) = inks.get_mut(&showcase.ink_material) {
            ink.animation = Vec4::new(
                smooth(local / 0.65) * 1.13 - 0.065,
                if local > 2.45 {
                    (local - 2.45) / 0.52 * 1.15
                } else {
                    -0.1
                },
                entry * (1.0 - smooth((local - 2.72) / 0.4)),
                local,
            );
        }
        if let Ok((mut transform, _)) = parts.get_mut(showcase.ink) {
            transform.translation = Val2::px(((local - 0.35).max(0.0) * 1.8).sin() * 1.7, 0.0);
        }
        let title_entry = ease_out_cubic(((local - 0.24) / 0.34).clamp(0.0, 1.0));
        if let Ok((mut transform, _)) = parts.get_mut(showcase.title) {
            transform.translation =
                Val2::px(-14.0 * (1.0 - title_entry), -9.0 * (1.0 - title_entry));
            transform.scale = Vec2::splat(1.0 + 0.045 * (1.0 - title_entry));
        }
        if let Ok(mut title) = images.get_mut(showcase.title) {
            title.color = Color::WHITE.with_alpha(smooth((local - 0.24) / 0.18) * entry * fade);
        }
        if let Ok(mut name) = names.get_mut(showcase.name) {
            name.0
                .set_alpha(smooth((local - 0.38) / 0.2) * entry * fade);
        }
    }
}
