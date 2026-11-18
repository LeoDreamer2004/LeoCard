use super::super::palette::{self, ColorPicker};
use super::super::state::{CanvasPaper, DrawGuessUiState};
use bevy::camera::RenderTarget;
use bevy::ecs::system::RunSystemOnce;
use bevy::picking::hover::PickingInteraction;
use bevy::picking::pointer::{Location, PointerId, PointerLocation};
use bevy::prelude::*;
use bevy::ui::picking_backend::UiPickingSettings;
use bevy::window::PrimaryWindow;

pub(super) fn assert_drawing_surfaces_receive_the_pointer(app: &mut App, target: &RenderTarget) {
    assert!(app.world().resource::<UiPickingSettings>().require_markers);
    let mut regions = Vec::new();
    for (entity, node, transform) in app
        .world_mut()
        .query_filtered::<(Entity, &ComputedNode, &UiGlobalTransform), With<CanvasPaper>>()
        .iter(app.world())
    {
        for x in [0.2, 0.8] {
            regions.push((
                entity,
                transform.transform_point2(node.size() * Vec2::new(x - 0.5, -0.2)),
                None,
            ));
        }
    }
    for (entity, kind, node, transform) in app
        .world_mut()
        .query::<(Entity, &ColorPicker, &ComputedNode, &UiGlobalTransform)>()
        .iter(app.world())
    {
        regions.push((
            entity,
            transform.transform_point2(node.size() * Vec2::new(-0.25, 0.0)),
            Some(*kind),
        ));
    }
    assert_eq!(regions.len(), 4);
    let pointer = app.world_mut().spawn(PointerId::Touch(u64::MAX)).id();
    for (entity, position, picker) in regions {
        app.world_mut()
            .entity_mut(pointer)
            .insert(PointerLocation::new(Location {
                target: target.normalize(None).unwrap(),
                position,
            }));
        for _ in 0..3 {
            app.update();
        }
        assert_ne!(
            *app.world().get::<PickingInteraction>(entity).unwrap(),
            PickingInteraction::None,
            "drawing surface must participate in the application's marker-only picking"
        );
        if let Some(kind) = picker {
            let mut window = Window {
                focused: true,
                ..default()
            };
            window.set_cursor_position(Some(position));
            let window = app.world_mut().spawn((window, PrimaryWindow)).id();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.world_mut()
                .run_system_once(palette::update_picker)
                .unwrap();
            let ui = app.world().resource::<DrawGuessUiState>();
            let value = match kind {
                ColorPicker::Square => ui.saturation,
                ColorPicker::Hue => ui.hue,
            };
            assert!((value - 0.25).abs() < 0.01);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            app.world_mut().despawn(window);
        }
    }
    app.world_mut().despawn(pointer);
}
