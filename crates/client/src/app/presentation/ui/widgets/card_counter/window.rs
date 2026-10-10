use bevy::{
    ecs::component::Mutable, prelude::*, ui::RelativeCursorPosition, window::PrimaryWindow,
};
use std::marker::PhantomData;

pub(crate) struct CardCounterWindowState {
    pub drawer_open: bool,
    pub enabled: bool,
    pub active: bool,
    pub position: Vec2,
    pub grab: Option<Vec2>,
}

impl Default for CardCounterWindowState {
    fn default() -> Self {
        Self {
            drawer_open: false,
            enabled: false,
            active: false,
            position: Vec2::new(34.0, 296.0),
            grab: None,
        }
    }
}

pub(crate) trait CardCounterOwner: Resource<Mutability = Mutable> {
    fn counter_window(&mut self) -> &mut CardCounterWindowState;
}

#[derive(Component)]
pub(crate) struct CounterWindow<T: CardCounterOwner>(pub PhantomData<T>);

#[derive(Component)]
pub(crate) struct CounterDragHandle<T: CardCounterOwner>(pub PhantomData<T>);

pub(crate) fn drag_card_counter_window<T: CardCounterOwner>(
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    scale: Res<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    handles: Query<&RelativeCursorPosition, With<CounterDragHandle<T>>>,
    mut panels: Query<(&mut Node, &ComputedNode), With<CounterWindow<T>>>,
    mut ui: ResMut<T>,
) {
    let state = ui.counter_window();
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
