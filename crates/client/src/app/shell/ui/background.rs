//! A continuous ambient backdrop shared by all pages outside a match.

use crate::app::presentation::spawn_node;
use crate::app::runtime::{ClientUpdateSet, UiAssets};
use bevy::prelude::*;
use bevy::ui::{
    BackgroundGradient, ColorStop, FocusPolicy, Gradient, LinearGradient, RadialGradient,
    RadialGradientShape, UiPosition,
};

pub(crate) struct AmbientBackgroundPlugin;

impl Plugin for AmbientBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_background.in_set(ClientUpdateSet::Animate));
    }
}

#[derive(Component)]
struct AmbientLight(usize);

// Two permanent purple lights and three softer visitors. Different periods keep
// their paths and breathing from moving in sync.
const LIGHTS: [(Color, f32); 5] = [
    (Color::srgb(0.30, 0.24, 0.45), 0.24),
    (Color::srgb(0.54, 0.42, 0.82), 0.12),
    (Color::srgb(0.09, 0.28, 0.32), 0.085),
    (Color::srgb(0.46, 0.30, 0.56), 0.065),
    (Color::srgb(0.22, 0.28, 0.48), 0.075),
];

pub(crate) fn add_page_background(commands: &mut Commands, root: Entity, assets: &UiAssets) {
    let canvas = spawn_node(
        commands,
        root,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(Color::srgb(0.045, 0.05, 0.075)),
    );
    commands.entity(canvas).insert((
        FocusPolicy::Pass,
        BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_bottom_right(
            vec![
                ColorStop::percent(Color::srgb(0.11, 0.10, 0.16), 0.0),
                ColorStop::percent(Color::srgb(0.075, 0.078, 0.12), 48.0),
                ColorStop::percent(Color::srgb(0.038, 0.055, 0.078), 100.0),
            ],
        ))]),
    ));
    for (index, (color, strength)) in LIGHTS.into_iter().enumerate() {
        let (position, opacity) = light_sample(index, 0.0);
        let light = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(-50),
                    top: percent(-50),
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                BackgroundGradient(vec![Gradient::Radial(RadialGradient::new(
                    UiPosition::CENTER,
                    RadialGradientShape::Ellipse(percent(50), percent(50)),
                    vec![
                        ColorStop::percent(color.with_alpha(strength * opacity), 0.0),
                        ColorStop::percent(color.with_alpha(0.0), 100.0),
                    ],
                ))]),
                UiTransform::from_translation(Val2::percent(position.x, position.y)),
                AmbientLight(index),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(canvas).add_child(light);
    }
    let felt = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            ImageNode::new(assets.table_felt.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgba(0.45, 0.45, 0.70, 0.08)),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(canvas).add_child(felt);
}

fn light_sample(index: usize, seconds: f32) -> (Vec2, f32) {
    if index < 2 {
        let phase = seconds * if index == 0 { 0.075 } else { 0.062 };
        let offset = index as f32 * 2.1;
        let center = if index == 0 {
            Vec2::new(74.0, 26.0)
        } else {
            Vec2::new(26.0, 74.0)
        };
        return (
            center
                + Vec2::new(
                    26.0 * (phase + offset).sin() + 6.0 * (phase * 1.61 + offset).sin(),
                    20.0 * (phase * 0.79 + offset).cos() + 5.0 * (phase * 1.37 + offset).sin(),
                ),
            1.0 + 0.18 * (phase * 1.7 + offset).sin(),
        );
    }
    let visitor = (index - 2) as f32;
    let cycle = (seconds / (85.0 + visitor * 23.0) + visitor * 0.31).fract();
    let progress = (cycle / 0.72).min(1.0);
    let x = if index == 3 {
        125.0 - 150.0 * progress
    } else {
        -25.0 + 150.0 * progress
    };
    (
        Vec2::new(
            x,
            52.0 + 32.0 * (progress * std::f32::consts::PI + visitor * 1.8).sin()
                + 7.0 * (progress * std::f32::consts::TAU + visitor).sin(),
        ),
        (progress * std::f32::consts::PI).sin().powi(2),
    )
}

fn animate_background(
    time: Res<Time>,
    mut lights: Query<(&AmbientLight, &mut UiTransform, &mut BackgroundGradient)>,
) {
    for (light, mut transform, mut gradient) in &mut lights {
        // Global time survives UI rebuilds and page transitions.
        let (position, opacity) = light_sample(light.0, time.elapsed_secs());
        *transform = UiTransform::from_translation(Val2::percent(position.x, position.y));
        if let Gradient::Radial(radial) = &mut gradient.0[0] {
            let (color, strength) = LIGHTS[light.0];
            radial.stops[0].color = color.with_alpha(strength * opacity);
        }
    }
}
