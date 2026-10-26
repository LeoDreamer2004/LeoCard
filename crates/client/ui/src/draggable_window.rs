use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
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
pub struct WindowDragHandle<T: DraggableWindowOwner>(pub PhantomData<T>);

pub fn drag_draggable_window<T: DraggableWindowOwner>(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    handles: Query<&RelativeCursorPosition, With<WindowDragHandle<T>>>,
    mut panels: Query<(&mut Node, &ComputedNode), With<DraggableWindow<T>>>,
    mut ui: ResMut<T>,
) {
    let state = ui.window_state();
    if !mouse.pressed(MouseButton::Left) || panels.is_empty() {
        state.grab = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window
        .cursor_position()
        .or_else(|| touches.first_pressed_position())
    else {
        return;
    };
    let cursor = cursor / scale.0;
    if mouse.just_pressed(MouseButton::Left) && handles.iter().any(|handle| handle.cursor_over()) {
        state.grab = Some(cursor - state.position);
    }
    let Some(grab) = state.grab else {
        return;
    };
    for (mut node, computed) in &mut panels {
        let viewport = Vec2::new(window.width(), window.height()) / scale.0;
        let size = computed.size() * computed.inverse_scale_factor();
        state.position = (cursor - grab).clamp(Vec2::ZERO, (viewport - size).max(Vec2::ZERO));
        node.left = px(state.position.x);
        node.top = px(state.position.y);
    }
}
