use crate::TouchDragTarget;
use bevy::ecs::component::Mutable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::marker::PhantomData;

#[derive(Default)]
pub struct DraggableWindowState {
    pub position: Vec2,
    pub grab: Option<Vec2>,
}

pub trait DraggableWindowOwner: Resource<Mutability = Mutable> {
    fn window_state(&mut self) -> &mut DraggableWindowState;
}

#[derive(Component)]
pub struct DraggableWindow<T: DraggableWindowOwner>(pub PhantomData<T>);

#[derive(Component)]
#[require(PickingInteraction, Pickable, TouchDragTarget)]
pub struct WindowDragHandle<T: DraggableWindowOwner>(pub PhantomData<T>);

pub fn drag_draggable_window<T: DraggableWindowOwner>(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    handles: Query<&PickingInteraction, With<WindowDragHandle<T>>>,
    mut panels: Query<(&mut Node, &ComputedNode), With<DraggableWindow<T>>>,
    mut ui: ResMut<T>,
) {
    let state = ui.window_state();
    if panels.is_empty() {
        state.grab = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !mouse.pressed(MouseButton::Left) || !window.focused {
        state.grab = None;
    }
    let cursor = window
        .cursor_position()
        .or_else(|| touches.first_pressed_position())
        .map(|cursor| cursor / scale.0);
    if window.focused
        && mouse.just_pressed(MouseButton::Left)
        && handles
            .iter()
            .any(|handle| *handle == PickingInteraction::Pressed)
        && let Some(cursor) = cursor
    {
        state.grab = Some(cursor - state.position);
    }
    for (mut node, computed) in &mut panels {
        let viewport = Vec2::new(window.width(), window.height()) / scale.0;
        let size = computed.size() * computed.inverse_scale_factor();
        let position = cursor
            .zip(state.grab)
            .map_or(state.position, |(cursor, grab)| cursor - grab);
        state.position = position.clamp(Vec2::ZERO, (viewport - size).max(Vec2::ZERO));
        if node.left != px(state.position.x) || node.top != px(state.position.y) {
            node.left = px(state.position.x);
            node.top = px(state.position.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DraggableWindow, DraggableWindowOwner, DraggableWindowState, WindowDragHandle,
        drag_draggable_window,
    };
    use bevy::picking::hover::PickingInteraction;
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use std::marker::PhantomData;

    #[derive(Default, Resource)]
    struct FloatingPanel(DraggableWindowState);

    impl DraggableWindowOwner for FloatingPanel {
        fn window_state(&mut self) -> &mut DraggableWindowState {
            &mut self.0
        }
    }

    #[test]
    fn drag_respects_picking_scale_release_and_viewport_resize() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Touches>()
            .insert_resource(UiScale(2.0))
            .insert_resource(FloatingPanel(DraggableWindowState {
                position: Vec2::new(10.0, 20.0),
                ..default()
            }))
            .add_systems(Update, drag_draggable_window::<FloatingPanel>);
        let mut window = Window {
            resolution: (800, 600).into(),
            ..default()
        };
        window.set_cursor_position(Some(Vec2::new(30.0, 40.0)));
        let window = app.world_mut().spawn((window, PrimaryWindow)).id();
        app.world_mut().spawn((
            DraggableWindow::<FloatingPanel>(PhantomData),
            Node::default(),
            ComputedNode {
                size: Vec2::new(200.0, 100.0),
                inverse_scale_factor: 0.5,
                ..default()
            },
        ));
        let handle = app
            .world_mut()
            .spawn(WindowDragHandle::<FloatingPanel>(PhantomData))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert!(app.world().resource::<FloatingPanel>().0.grab.is_none());

        app.world_mut()
            .entity_mut(handle)
            .insert(PickingInteraction::Pressed);
        app.update();
        assert_eq!(
            app.world().resource::<FloatingPanel>().0.grab,
            Some(Vec2::new(5.0, 0.0))
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::new(790.0, 590.0)));
        app.update();
        assert_eq!(
            app.world().resource::<FloatingPanel>().0.position,
            Vec2::new(300.0, 250.0)
        );

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .set_cursor_position(Some(Vec2::ZERO));
        app.update();
        assert!(app.world().resource::<FloatingPanel>().0.grab.is_none());
        assert_eq!(
            app.world().resource::<FloatingPanel>().0.position,
            Vec2::new(300.0, 250.0)
        );
        app.world_mut()
            .get_mut::<Window>(window)
            .unwrap()
            .resolution
            .set(400.0, 300.0);
        app.update();
        assert_eq!(
            app.world().resource::<FloatingPanel>().0.position,
            Vec2::new(100.0, 100.0)
        );
    }
}
