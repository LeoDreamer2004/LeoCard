use crate::app::presentation::add_text;
use crate::app::runtime::UiAssets;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_ui::UiComposer;
use std::f32::consts;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum TableStatusEffect {
    #[default]
    Appear,
    Settle,
    Dim,
    Pulse,
    Bounce,
    Impact,
}

pub(crate) struct TableStatusSpec<'a> {
    pub text: &'a str,
    pub font_size: f32,
    pub color: Color,
    pub effect: TableStatusEffect,
    pub elapsed: Option<f32>,
}

#[derive(Component, Clone, Default, PartialEq)]
pub(crate) struct TableStatusAnimation {
    text: String,
    color: Color,
    effect: TableStatusEffect,
    elapsed: f32,
}

pub(crate) struct TableStatusVisual {
    pub translation: Vec2,
    scale: f32,
    alpha: f32,
}

impl TableStatusEffect {
    pub(crate) fn visual(self, elapsed: f32) -> TableStatusVisual {
        let entry = (elapsed / 0.18).clamp(0.0, 1.0);
        let mut visual = TableStatusVisual {
            translation: Vec2::ZERO,
            scale: 0.84 + entry * 0.16,
            alpha: entry,
        };
        match self {
            Self::Appear | Self::Dim => {}
            Self::Settle => visual.scale = 0.90 + entry * 0.10,
            Self::Pulse => visual.scale = 1.0 + (entry * consts::PI).sin() * 0.12,
            Self::Bounce => {
                let progress = (elapsed / 0.28).clamp(0.0, 1.0);
                visual.scale = 0.78 + 1.22 * progress - progress * progress;
            }
            Self::Impact => {
                let progress = (elapsed / 0.34).clamp(0.0, 1.0);
                visual.scale = 0.72 + 1.40 * progress - 1.12 * progress * progress;
                let shake = 1.0 - (elapsed / 0.48).clamp(0.0, 1.0);
                visual.translation.x = (elapsed * 56.0).sin() * 5.5 * shake;
            }
        }
        visual
    }

    fn text_color(self, base: Color, elapsed: f32) -> Color {
        let alpha = self.visual(elapsed).alpha;
        if self != Self::Dim {
            return base.with_alpha(alpha);
        }
        let progress = ((elapsed - 0.22) / 0.42).clamp(0.0, 1.0);
        let eased = progress * progress * (3.0 - 2.0 * progress);
        let rgb = Vec3::new(0.94, 0.97, 0.95).lerp(Vec3::new(0.50, 0.55, 0.53), eased);
        Color::srgba(rgb.x, rgb.y, rgb.z, alpha)
    }
}

impl TableStatusVisual {
    fn transform(&self) -> UiTransform {
        UiTransform {
            translation: Val2::px(self.translation.x, self.translation.y),
            scale: Vec2::splat(self.scale),
            ..default()
        }
    }
}

pub(crate) fn add_table_status(
    commands: &mut UiComposer,
    parent: Entity,
    spec: TableStatusSpec<'_>,
    assets: &UiAssets,
) -> Entity {
    let elapsed = spec.elapsed.unwrap_or(0.0);
    let text = add_text(
        commands,
        parent,
        spec.text,
        spec.font_size,
        spec.effect.text_color(spec.color, elapsed),
        assets,
    );
    let changed = commands
        .declared::<TableStatusAnimation>(text)
        .is_none_or(|before| {
            before.text != spec.text || before.color != spec.color || before.effect != spec.effect
        });
    let animation = TableStatusAnimation {
        text: spec.text.to_owned(),
        color: spec.color,
        effect: spec.effect,
        elapsed,
    };
    commands.entity(text).insert((
        spec.effect.visual(elapsed).transform(),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    ));
    if changed || spec.elapsed.is_some() {
        commands.entity(text).insert(animation);
    } else {
        commands
            .entity(text)
            .entry::<TableStatusAnimation>()
            .or_insert(animation);
    }
    text
}

pub(crate) fn animate_table_statuses(
    time: Res<Time>,
    mut labels: Query<(&mut TableStatusAnimation, &mut UiTransform, &mut TextColor)>,
) {
    for (mut animation, mut transform, mut color) in &mut labels {
        animation.elapsed += time.delta_secs();
        *transform = animation.effect.visual(animation.elapsed).transform();
        color.0 = animation
            .effect
            .text_color(animation.color, animation.elapsed);
    }
}
