use super::spawn_node;
use crate::UiComposer;
use bevy::prelude::*;

/// `None` indicates work whose total size is unknown.
#[derive(Component)]
pub struct ProgressBar {
    pub fraction: Option<f32>,
}

impl ProgressBar {
    pub fn spawn(commands: &mut UiComposer, parent: Entity, fraction: Option<f32>) -> Entity {
        let track = spawn_node(
            commands,
            parent,
            Node {
                width: percent(100),
                height: px(14),
                flex_shrink: 0.0,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                overflow: Overflow::clip(),
                ..default()
            },
            Some(Color::srgb(0.15, 0.15, 0.18)),
        );
        commands
            .entity(track)
            .insert(BorderColor::all(Color::srgba(0.62, 0.60, 0.69, 0.52)));
        let fill = spawn_node(
            commands,
            track,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(fraction.map_or(28.0, |fraction| fraction.clamp(0.0, 1.0) * 100.0)),
                height: percent(100),
                ..default()
            },
            Some(Color::srgb(0.54, 0.49, 0.88)),
        );
        commands.entity(fill).insert(Self { fraction });
        fill
    }
}

pub fn animate_progress_bars(time: Res<Time>, mut bars: Query<(&ProgressBar, &mut Node)>) {
    for (bar, mut node) in &mut bars {
        let (width, left) = match bar.fraction {
            Some(fraction) => (percent(fraction.clamp(0.0, 1.0) * 100.0), px(0)),
            None => (
                percent(28),
                percent(((time.elapsed_secs() * 0.62) % 1.28 - 0.28) * 100.0),
            ),
        };
        if node.width != width || node.left != left {
            node.width = width;
            node.left = left;
        }
    }
}
