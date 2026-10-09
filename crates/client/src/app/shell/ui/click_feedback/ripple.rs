//! Short-lived click feedback, independent of page rebuilds and transitions.

use crate::app::runtime::ClientUpdateSet;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;

const DURATION: f32 = 0.4;
const START_RADIUS: f32 = 4.0;
const END_RADIUS: f32 = 22.0;
const OPACITY: f32 = 0.32;
const COLOR: Color = Color::srgb(0.90, 0.87, 1.0);
const Z_INDEX: i32 = 3000;

pub(crate) struct ClickFeedbackPlugin;

impl Plugin for ClickFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_ripples, spawn_ripple)
                .chain()
                .in_set(ClientUpdateSet::Animate),
        );
    }
}

#[derive(Component)]
struct ClickRipple {
    cursor: Vec2,
    timer: Timer,
}

impl ClickRipple {
    fn new(cursor: Vec2) -> Self {
        Self {
            cursor,
            timer: Timer::from_seconds(DURATION, TimerMode::Once),
        }
    }

    fn node(&self, scale: f32) -> Node {
        let expansion = 1.0 - (1.0 - self.timer.fraction()).powi(3);
        let radius = (START_RADIUS + (END_RADIUS - START_RADIUS) * expansion) / scale;
        // Cursor coordinates are window logical pixels; UI lengths also include UiScale.
        let center = self.cursor / scale;
        Node {
            position_type: PositionType::Absolute,
            left: px(center.x - radius),
            top: px(center.y - radius),
            width: px(radius * 2.0),
            height: px(radius * 2.0),
            border: UiRect::all(px(1.2 / scale)),
            border_radius: BorderRadius::MAX,
            ..default()
        }
    }

    fn color(&self) -> BorderColor {
        BorderColor::all(COLOR.with_alpha(OPACITY * (1.0 - self.timer.fraction()).powi(2)))
    }
}

fn spawn_ripple(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    scale: Res<UiScale>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cursor) = windows.single().ok().and_then(Window::cursor_position) else {
        return;
    };
    let ripple = ClickRipple::new(cursor);
    // A separate root survives page rebuilds; ignore hits in both UI input systems.
    commands.spawn((
        ripple.node(scale.0),
        ripple.color(),
        ripple,
        GlobalZIndex(Z_INDEX),
        FocusPolicy::Pass,
        Pickable::IGNORE,
    ));
}

fn animate_ripples(
    mut commands: Commands,
    time: Res<Time>,
    scale: Res<UiScale>,
    mut ripples: Query<(Entity, &mut ClickRipple, &mut Node, &mut BorderColor)>,
) {
    for (entity, mut ripple, mut node, mut color) in &mut ripples {
        ripple.timer.tick(time.delta());
        if ripple.timer.is_finished() {
            commands.entity(entity).despawn();
        } else {
            *node = ripple.node(scale.0);
            *color = ripple.color();
        }
    }
}
