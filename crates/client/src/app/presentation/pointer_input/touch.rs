//! One primary finger drives the existing mouse pointer, so every game shares drag and hover logic.
//! Scroll gestures defer presses until release, allowing category lists to swipe without selecting tabs.

use super::{super::CardDragSelection, SecondaryPressTarget};
use bevy::{
    ecs::system::SystemParam,
    input::{
        ButtonState,
        mouse::MouseButtonInput,
        touch::{TouchInput, TouchPhase},
    },
    picking::{PickingSystems, input::PointerInputSettings},
    prelude::*,
    window::{CursorMoved, WindowEvent},
};

pub(super) fn install(app: &mut App) {
    app.insert_resource(PointerInputSettings {
        is_touch_enabled: false,
        is_mouse_enabled: true,
    })
    .init_resource::<PrimaryTouch>()
    .add_systems(First, adapt_touch.before(PickingSystems::Input));
}

#[derive(Resource, Default)]
struct PrimaryTouch {
    finger: Option<u64>,
    origin: Vec2,
    last: Vec2,
    scroll: Option<Entity>,
    scrolling: bool,
    secondary: bool,
    held: bool,
    started_at: f64,
    window: Option<Entity>,
}

#[derive(SystemParam)]
struct TouchOutput<'w, 's> {
    windows: Query<'w, 's, &'static Window>,
    scrolls: Query<
        'w,
        's,
        (
            Entity,
            &'static Node,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static mut ScrollPosition,
        ),
    >,
    secondary_regions: Query<
        'w,
        's,
        (&'static ComputedNode, &'static UiGlobalTransform),
        With<SecondaryPressTarget>,
    >,
    window_events: MessageWriter<'w, WindowEvent>,
    buttons: MessageWriter<'w, MouseButtonInput>,
    card_drag: ResMut<'w, CardDragSelection>,
}

impl TouchOutput<'_, '_> {
    fn move_pointer(&mut self, event: &TouchInput, delta: Vec2) {
        self.window_events
            .write(WindowEvent::CursorMoved(CursorMoved {
                window: event.window,
                position: event.position,
                delta: Some(delta),
            }));
    }

    fn button(&mut self, window: Entity, button: MouseButton, state: ButtonState) {
        let event = MouseButtonInput {
            button,
            state,
            window,
        };
        self.window_events
            .write(WindowEvent::MouseButtonInput(event));
        self.buttons.write(event);
    }

    fn scroll_at(&mut self, event: &TouchInput) -> Option<Entity> {
        let scale = self.windows.get(event.window).ok()?.scale_factor();
        let position = event.position * scale;
        self.scrolls
            .iter()
            .find(|(_, style, node, transform, _)| {
                style.overflow.y == OverflowAxis::Scroll
                    && node.content_size().y > node.size().y
                    && transform.try_inverse().is_some_and(|inverse| {
                        let local = inverse.transform_point2(position);
                        local.abs().cmple(node.size() * 0.5).all()
                    })
            })
            .map(|(entity, _, _, _, _)| entity)
    }
}

fn adapt_touch(
    mut events: MessageReader<TouchInput>,
    mut touch: ResMut<PrimaryTouch>,
    mut output: TouchOutput,
    time: Res<Time<Real>>,
) {
    for event in events.read() {
        if event.phase == TouchPhase::Started && touch.finger.is_none() {
            touch.finger = Some(event.id);
            touch.started_at = time.elapsed_secs_f64();
            touch.window = Some(event.window);
            touch.held = false;
            let scale = output
                .windows
                .get(event.window)
                .map_or(1.0, Window::scale_factor);
            touch.secondary = output.secondary_regions.iter().any(|(node, transform)| {
                transform.try_inverse().is_some_and(|inverse| {
                    inverse
                        .transform_point2(event.position * scale)
                        .abs()
                        .cmple(node.size() * 0.5)
                        .all()
                })
            });
            touch.origin = event.position;
            touch.last = event.position;
            touch.scrolling = false;
            touch.scroll = output.scroll_at(event);
            output.move_pointer(event, Vec2::ZERO);
            if touch.scroll.is_none() && !touch.secondary {
                output.button(event.window, MouseButton::Left, ButtonState::Pressed);
            }
            continue;
        }
        if touch.finger != Some(event.id) {
            continue;
        }
        let delta = event.position - touch.last;
        output.move_pointer(event, delta);
        match event.phase {
            TouchPhase::Moved => {
                touch.scrolling |= event.position.distance(touch.origin) > 8.0;
                if touch.scrolling
                    && let Some(entity) = touch.scroll
                    && let Ok((_, _, node, _, mut position)) = output.scrolls.get_mut(entity)
                {
                    let maximum = ((node.content_size().y - node.size().y)
                        * node.inverse_scale_factor())
                    .max(0.0);
                    let scale = output
                        .windows
                        .get(event.window)
                        .map_or(1.0, Window::scale_factor);
                    position.y = (position.y - delta.y * scale * node.inverse_scale_factor())
                        .clamp(0.0, maximum);
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if event.phase == TouchPhase::Canceled {
                    output.card_drag.active = false;
                }
                if touch.scroll.is_none() && !touch.secondary {
                    output.button(event.window, MouseButton::Left, ButtonState::Released);
                } else if !touch.scrolling && !touch.held && event.phase == TouchPhase::Ended {
                    output.button(event.window, MouseButton::Left, ButtonState::Pressed);
                    output.button(event.window, MouseButton::Left, ButtonState::Released);
                }
                touch.finger = None;
            }
            TouchPhase::Started => {}
        }
        touch.last = event.position;
    }
    if touch.finger.is_some()
        && touch.secondary
        && !touch.held
        && !touch.scrolling
        && time.elapsed_secs_f64() - touch.started_at >= 0.55
        && let Some(window) = touch.window
    {
        output.button(window, MouseButton::Right, ButtonState::Pressed);
        output.button(window, MouseButton::Right, ButtonState::Released);
        touch.held = true;
    }
}
