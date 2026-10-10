//! Soft light layers use regular UI gradients so page transitions fade them too.

use super::ENTRY_DESIGNS;
use crate::app::presentation::TransitionOpacity;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, Gradient, RadialGradient, RadialGradientShape};

#[derive(Clone, Copy)]
pub(super) enum GlowKind {
    Aura,
    Orbit,
    Ripple,
    Underline,
}

impl GlowKind {
    fn profile(self) -> &'static [(f32, f32)] {
        match self {
            Self::Aura => &[(0.0, 0.24), (24.0, 0.15), (60.0, 0.045), (100.0, 0.0)],
            Self::Orbit => &[
                (0.0, 0.0),
                (68.0, 0.0),
                (73.0, 0.06),
                (78.0, 0.32),
                (82.0, 0.10),
                (90.0, 0.0),
                (100.0, 0.0),
            ],
            Self::Ripple => &[
                (0.0, 0.0),
                (71.0, 0.0),
                (80.0, 0.48),
                (88.0, 0.0),
                (100.0, 0.0),
            ],
            Self::Underline => &[(0.0, 0.65), (16.0, 0.34), (48.0, 0.10), (100.0, 0.0)],
        }
    }
}

#[derive(Component)]
pub(super) struct EntryGlow {
    pub index: usize,
    pub kind: GlowKind,
}

impl EntryGlow {
    pub fn spawn(commands: &mut Commands, parent: Entity, index: usize, kind: GlowKind) -> Entity {
        let (width, height, top) = match kind {
            GlowKind::Aura | GlowKind::Ripple => (112.0, 200.0, -2.0),
            GlowKind::Orbit => (100.0, 65.0, 135.0),
            GlowKind::Underline => (70.0, 10.0, 28.0),
        };
        let glow = Self { index, kind };
        let mut gradient = glow.gradient();
        if matches!(kind, GlowKind::Ripple) {
            glow.animate(&mut gradient, 0.0);
        }
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent((100.0 - width) * 0.5),
                    top: px(top),
                    width: percent(width),
                    height: px(height),
                    ..default()
                },
                gradient,
                TransitionOpacity::default(),
                glow,
                UiTransform::IDENTITY,
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(parent).add_child(entity);
        entity
    }

    fn gradient(&self) -> BackgroundGradient {
        let color = ENTRY_DESIGNS[self.index].accent;
        BackgroundGradient::from(RadialGradient::new(
            UiPosition::CENTER,
            RadialGradientShape::Ellipse(percent(50), percent(50)),
            self.kind
                .profile()
                .iter()
                .map(|&(position, alpha)| ColorStop::percent(color.with_alpha(alpha), position))
                .collect(),
        ))
    }

    pub fn animate(&self, gradient: &mut BackgroundGradient, strength: f32) {
        let Gradient::Radial(radial) = &mut gradient.0[0] else {
            return;
        };
        let color = ENTRY_DESIGNS[self.index].accent;
        for (stop, &(_, alpha)) in radial.stops.iter_mut().zip(self.kind.profile()) {
            stop.color = color.with_alpha(alpha * strength);
        }
    }
}
