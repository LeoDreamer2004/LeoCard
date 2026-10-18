use super::state::UnoDragGesture;
use super::{UnoDragGuideMaterial, UnoDragState};
use bevy::picking::Pickable;
use bevy::prelude::*;

#[derive(Component)]
pub(in super::super) struct UnoDragOverlay {
    guide: Entity,
    halo: Entity,
    material: Handle<UnoDragGuideMaterial>,
}

impl UnoDragOverlay {
    fn spawn(commands: &mut Commands, material: Handle<UnoDragGuideMaterial>) -> Self {
        let guide = commands
            .spawn((
                Node::default(),
                MaterialNode(material.clone()),
                Pickable::IGNORE,
            ))
            .id();
        let halo = commands
            .spawn((Node::default(), BoxShadow::default(), Pickable::IGNORE))
            .id();
        Self {
            guide,
            halo,
            material,
        }
    }
}

pub(in super::super) fn render_uno_drag(
    mut commands: Commands,
    drag: Res<UnoDragState>,
    time: Res<Time>,
    mut materials: ResMut<Assets<UnoDragGuideMaterial>>,
    overlays: Query<(Entity, &UnoDragOverlay)>,
) {
    let Some(gesture) = drag.gesture.as_ref().filter(|gesture| gesture.dragging) else {
        for (entity, _) in &overlays {
            commands.entity(entity).despawn();
        }
        return;
    };
    let start = gesture.origin;
    let end = gesture.cursor;
    let direction = (end - start).normalize_or_zero();
    let control = (start + end) * 0.5
        + Vec2::new(-direction.y, direction.x) * (start.distance(end) * 0.22).min(88.0);
    let min = start.min(control).min(end) - Vec2::splat(18.0);
    let max = start.max(control).max(end) + Vec2::splat(18.0);
    let size = max - min;
    let ready = gesture.ready();
    let color = if ready {
        Color::srgba(0.78, 0.74, 0.62, 0.58)
    } else {
        Color::srgba(0.65, 0.68, 0.72, 0.42)
    };
    let material = UnoDragGuideMaterial {
        start_control: Vec4::new(
            start.x - min.x,
            start.y - min.y,
            control.x - min.x,
            control.y - min.y,
        ),
        end_size: Vec4::new(end.x - min.x, end.y - min.y, size.x, size.y),
        color: color.to_linear(),
        phase: Vec4::new(time.elapsed_secs(), 0.0, 0.0, 0.0),
    };
    let (guide, halo) = if let Ok((_, overlay)) = overlays.single() {
        if let Some(mut current) = materials.get_mut(&overlay.material) {
            *current = material;
        }
        (overlay.guide, overlay.halo)
    } else {
        let overlay = UnoDragOverlay::spawn(&mut commands, materials.add(material));
        let parts = (overlay.guide, overlay.halo);
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                GlobalZIndex(1420),
                Pickable::IGNORE,
            ))
            .add_children(&[overlay.halo, overlay.guide])
            .insert(overlay);
        parts
    };
    commands.entity(guide).insert(Node {
        position_type: PositionType::Absolute,
        left: px(min.x),
        top: px(min.y),
        width: px(size.x),
        height: px(size.y),
        ..default()
    });
    let radius = UnoDragGesture::target_radius();
    commands.entity(halo).insert((
        Node {
            position_type: PositionType::Absolute,
            left: px(gesture.target.x - radius.x),
            top: px(gesture.target.y - radius.y),
            width: px(radius.x * 2.0),
            height: px(radius.y * 2.0),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        BoxShadow::new(
            color.with_alpha(if ready { 0.04 } else { 0.015 }),
            px(0),
            px(0),
            px(2),
            px(12),
        ),
    ));
}
