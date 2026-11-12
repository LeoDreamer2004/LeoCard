use super::super::timeline::{WinShowcaseTiming, smooth};
use super::impact::MajorGlyphImpact;
use super::{MahjongWinArtMaterial, MajorShowcaseSequence};
use crate::app::presentation::{GameSummaryAnimation, ease_out_cubic};
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) struct MajorShowcaseGlyph {
    pub entity: Entity,
    pub material: Handle<MahjongWinArtMaterial>,
    pub bounds: Vec4,
}

#[derive(Component)]
pub(crate) struct MahjongMajorShowcase {
    pub(super) timing: WinShowcaseTiming,
    pub(super) viewport: Entity,
    pub(super) backdrop: Entity,
    pub(super) background: Handle<MahjongWinArtMaterial>,
    pub(super) glyphs: Vec<MajorShowcaseGlyph>,
}

pub(crate) fn animate_mahjong_major_showcases(
    animation: Res<GameSummaryAnimation>,
    showcases: Query<(Entity, &MahjongMajorShowcase)>,
    mut visibility: Query<&mut Visibility>,
    mut backdrops: Query<&mut BackgroundColor>,
    mut materials: ResMut<Assets<MahjongWinArtMaterial>>,
    mut nodes: Query<(&ComputedNode, &mut UiTransform)>,
) {
    let mut viewport_shakes = HashMap::<Entity, Vec3>::new();
    for (entity, showcase) in &showcases {
        let shake = viewport_shakes.entry(showcase.viewport).or_default();
        let local = showcase.timing.local(animation.elapsed);
        let active = showcase.timing.active(local);
        if let Ok(mut visible) = visibility.get_mut(entity) {
            *visible = if active {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if !active {
            continue;
        }
        let fade =
            1.0 - smooth((local - MajorShowcaseSequence::EXIT) / MajorShowcaseSequence::FADE);
        if let Ok(mut backdrop) = backdrops.get_mut(showcase.backdrop) {
            backdrop.0 = Color::BLACK.with_alpha(0.65 * smooth(local / 0.3) * fade);
        }
        if let Some(mut material) = materials.get_mut(&showcase.background) {
            let entry = smooth(local / 0.5);
            material.animation = Vec4::new(
                smooth(local / 0.2) * fade,
                entry * fade,
                1.0 + 0.035 * (1.0 - ease_out_cubic((local / 0.6).clamp(0.0, 1.0))),
                0.0,
            );
        }
        for (index, glyph) in showcase.glyphs.iter().enumerate() {
            let impact = MajorGlyphImpact::at(local, index);
            *shake += impact.shake();
            if let Some(mut material) = materials.get_mut(&glyph.material) {
                material.animation = Vec4::new(smooth(impact.age / 0.035) * fade, 1.0, 1.0, 1.0);
                let hold_elapsed = local - MajorShowcaseSequence::HOLD_START;
                material.pose = Vec4::new(
                    0.0,
                    0.0,
                    1.0,
                    -0.35 + hold_elapsed / MajorShowcaseSequence::POLISH_DURATION * 1.7,
                );
                material.surface.y = smooth(hold_elapsed / 0.15)
                    * (1.0 - smooth((local - MajorShowcaseSequence::EXIT + 0.15) / 0.15));
                if let Ok((node, mut transform)) = nodes.get_mut(glyph.entity) {
                    *transform = impact.transform(
                        node.size() * node.inverse_scale_factor(),
                        material.surface.x,
                        glyph.bounds,
                    );
                }
            }
        }
    }
    // Inactive winner stages must not reset another winner's current screen impact.
    for (viewport, shake) in viewport_shakes {
        if let Ok((_, mut transform)) = nodes.get_mut(viewport) {
            *transform = UiTransform {
                translation: Val2::px(shake.x, shake.y),
                rotation: Rot2::radians(shake.z),
                ..default()
            };
        }
    }
}
