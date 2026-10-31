use super::AnchoredBubble;
use super::systems::animate_bubbles;
use bevy::prelude::*;
use std::time::Duration;

fn advance(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    app.update();
}

#[test]
fn bubble_tracks_a_replacement_anchor_and_tolerates_a_brief_rebuild_gap() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, animate_bubbles);
    let layer = app
        .world_mut()
        .spawn((
            ComputedNode {
                size: Vec2::new(400.0, 300.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::default(),
        ))
        .id();
    let anchor = app
        .world_mut()
        .spawn((
            ComputedNode {
                size: Vec2::splat(50.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(-100.0, 0.0)),
        ))
        .id();
    let bubble = app
        .world_mut()
        .spawn((
            ChildOf(layer),
            AnchoredBubble {
                anchor: Some(anchor),
                anchor_offset: Vec2::ZERO,
                text: None,
                image: None,
                size: Vec2::splat(86.0),
                elapsed: 0.0,
                duration: 3.4,
                missing_secs: 0.0,
                background_color: Color::WHITE,
                text_color: Color::WHITE,
            },
            Node::default(),
            UiTransform::default(),
            ImageNode::default(),
            Visibility::Inherited,
        ))
        .id();
    advance(&mut app, 0.1);
    let previous = app.world().get::<Node>(bubble).unwrap().left;
    app.world_mut().despawn(anchor);
    advance(&mut app, 0.1);
    assert_eq!(app.world().get::<Node>(bubble).unwrap().left, previous);
    assert!(
        app.world()
            .get::<AnchoredBubble>(bubble)
            .unwrap()
            .missing_secs
            > 0.0
    );

    let replacement = app
        .world_mut()
        .spawn((
            ComputedNode {
                size: Vec2::splat(50.0),
                inverse_scale_factor: 1.0,
                ..default()
            },
            UiGlobalTransform::from_translation(Vec2::new(100.0, 0.0)),
        ))
        .id();
    app.world_mut()
        .get_mut::<AnchoredBubble>(bubble)
        .unwrap()
        .anchor = Some(replacement);
    advance(&mut app, 0.1);
    assert_ne!(app.world().get::<Node>(bubble).unwrap().left, previous);
    assert_eq!(
        app.world()
            .get::<AnchoredBubble>(bubble)
            .unwrap()
            .missing_secs,
        0.0
    );

    app.world_mut().despawn(replacement);
    advance(&mut app, 0.6);
    assert!(app.world().get_entity(bubble).is_err());
}
