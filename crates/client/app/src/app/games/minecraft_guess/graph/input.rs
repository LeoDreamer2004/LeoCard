use super::super::MinecraftGuessUiState;
use super::super::feedback::FeedbackSound;
use super::guide::GraphGuideState;
use super::log::GraphLogRoot;
use super::state::{Gesture, GraphCanvas, GraphDragTarget, GraphWorkspace};
use crate::app::runtime::ClientResource;
use crate::app::shell::{ConfirmationDialog, PageMotion, PlayErrorToast, UiState, game_command};
use bevy::camera::NormalizedRenderTarget;
use bevy::ecs::system::SystemParam;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
#[cfg(target_os = "android")]
use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::picking::events::PointerPress;
use bevy::picking::pointer::{Location, PointerAction, PointerButton, PointerInput};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_protocol::MinecraftGuessCommand;

#[derive(SystemParam)]
pub(in super::super) struct GraphPointerContext<'w, 's> {
    client: Option<Res<'w, ClientResource>>,
    state: ResMut<'w, MinecraftGuessUiState>,
    ui: ResMut<'w, UiState>,
    toast: ResMut<'w, PlayErrorToast>,
    sounds: MessageWriter<'w, FeedbackSound>,
    confirmation: Res<'w, ConfirmationDialog>,
    motion: Res<'w, PageMotion>,
    guide: Res<'w, GraphGuideState>,
    logs: Query<'w, 's, &'static RelativeCursorPosition, With<GraphLogRoot>>,
    targets: Query<'w, 's, &'static GraphDragTarget>,
    canvas: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static RelativeCursorPosition,
        ),
        With<GraphCanvas>,
    >,
    workspace:
        Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<GraphWorkspace>>,
    #[cfg(target_os = "android")]
    touches: MessageReader<'w, 's, TouchInput>,
    windows: Query<'w, 's, &'static Window>,
}

impl GraphPointerContext<'_, '_> {
    fn select_node(&mut self, number: u16) {
        let blocked = self.client.as_deref().is_some_and(|client| {
            client.0.model().minecraft_guess_game().is_some_and(|game| {
                game.round.can_guess(game.you.0, game.rules.mode)
                    && game
                        .graph
                        .as_ref()
                        .is_some_and(|graph| graph.combo.is_some() && !graph.can_guess_node(number))
            })
        });
        if blocked {
            self.toast.show_message("此节点不可连击".to_owned());
            self.sounds.write(FeedbackSound::ComboBlocked);
            self.ui.dirty = true;
            return;
        }
        if self.state.graph.selected != Some(number) {
            self.state.graph.selected = Some(number);
            self.sounds.write(FeedbackSound::NodeSelect);
        }
    }

    fn blocked(&self) -> bool {
        self.ui.settings.open
            || !self.windows.iter().any(|window| window.focused)
            || self.state.graph.positions.is_empty()
            || self.logs.iter().any(|cursor| cursor.cursor_over())
            || self.guide.visible()
            || self.ui.shop.open
            || self.ui.profile.open
            || self.ui.leaving_room
            || self.confirmation.is_open()
            || self.motion.active()
            || !self.client.as_deref().is_some_and(|c| {
                c.0.model()
                    .minecraft_guess_game()
                    .is_some_and(|g| g.graph.is_some())
            })
    }

    fn point(&self, target: GraphDragTarget, location: &Location) -> Option<(Vec2, Vec2)> {
        let NormalizedRenderTarget::Window(window) = location.target else {
            return None;
        };
        let window = self.windows.get(window.entity()).ok()?;
        if !window.focused {
            return None;
        }
        let (node, transform) = match target {
            GraphDragTarget::Divider => self.workspace.single().ok()?,
            _ => {
                let (node, transform, _) = self.canvas.single().ok()?;
                (node, transform)
            }
        };
        if node.size().min_element() <= 0.0 {
            return None;
        }
        let point = transform
            .affine()
            .inverse()
            .transform_point2(location.position * window.scale_factor())
            + node.size() * 0.5;
        Some((
            point * node.inverse_scale_factor(),
            node.size() * node.inverse_scale_factor(),
        ))
    }
}

pub(in super::super) fn sync_graph_selection(
    mut client: Option<ResMut<ClientResource>>,
    mut state: ResMut<MinecraftGuessUiState>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(number) = state.graph.selected else {
        return;
    };
    // Publish the local selection on turn changes as well as pointer selections.
    // Other players keep browsing independently without changing the active question.
    if state.graph.selection_wait <= 0.0
        && client.0.model().minecraft_guess_game().is_some_and(|game| {
            game.round.current_player == Some(game.you.0)
                && !game.round.finished()
                && game.graph.as_ref().is_some_and(|graph| {
                    graph.selected != number
                        && (graph.combo.is_none() || graph.can_guess_node(number))
                })
        })
        && client
            .0
            .send(game_command(MinecraftGuessCommand::SelectNode { number }))
    {
        state.graph.selection_wait = 0.6;
    }
}

pub(in super::super) fn graph_pointer_input(
    time: Res<Time>,
    mut presses: MessageReader<PointerPress>,
    mut inputs: MessageReader<PointerInput>,
    mut wheels: MessageReader<MouseWheel>,
    mut context: GraphPointerContext,
) {
    #[cfg(target_os = "android")]
    if context
        .touches
        .read()
        .any(|touch| touch.phase == TouchPhase::Canceled)
    {
        context.state.graph.gesture = None;
        context.state.graph.camera.stop();
        presses.clear();
        inputs.clear();
        wheels.clear();
        return;
    }
    if context.blocked() {
        context.state.graph.gesture = None;
        context.state.graph.camera.stop();
        presses.clear();
        inputs.clear();
        wheels.clear();
        return;
    }
    let delta: f32 = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 0.13,
            MouseScrollUnit::Pixel => wheel.y * 0.002,
        })
        .sum();
    if delta != 0.0
        && context.state.graph.gesture.is_none()
        && let Ok((node, _, cursor)) = context.canvas.single()
        && cursor.cursor_over()
        && let Some(normalized) = cursor.normalized
    {
        let size = node.size() * node.inverse_scale_factor();
        context
            .state
            .graph
            .camera
            .zoom_at((normalized + Vec2::splat(0.5)) * size, size, delta);
    }
    let mut awaiting_press = false;
    for press in presses.read() {
        if press.button != PointerButton::Primary {
            continue;
        }
        let Ok(&target) = context.targets.get(press.entity) else {
            continue;
        };
        let Some((point, size)) = context.point(target, &press.pointer.location()) else {
            continue;
        };
        context.state.graph.camera.stop();
        let anchor = match target {
            GraphDragTarget::Node(number) => {
                context.state.graph.camera.unproject(point, size)
                    - context.state.graph.positions[usize::from(number - 1)]
            }
            _ => point,
        };
        context.state.graph.gesture = Some(Gesture {
            pointer: press.pointer.id,
            target,
            start: press.pointer.position,
            anchor,
            pan: context.state.graph.camera.pan,
            moved: false,
        });
        awaiting_press = true;
    }
    let before_pan = context.state.graph.camera.pan;
    let mut moved_pan = false;
    for input in inputs.read() {
        let Some(mut gesture) = context.state.graph.gesture.take() else {
            continue;
        };
        if input.pointer_id != gesture.pointer {
            context.state.graph.gesture = Some(gesture);
            continue;
        }
        if awaiting_press {
            awaiting_press = !input.button_just_pressed(PointerButton::Primary);
            context.state.graph.gesture = Some(gesture);
            continue;
        }
        gesture.moved |= gesture.start.distance(input.location.position) > 5.0;
        if matches!(
            input.action,
            PointerAction::Cancel | PointerAction::Press(PointerButton::Secondary)
        ) {
            context.state.graph.camera.stop();
            continue;
        }
        let Some((point, size)) = context.point(gesture.target, &input.location) else {
            continue;
        };
        if gesture.moved {
            match gesture.target {
                GraphDragTarget::Node(number) => {
                    let position =
                        context.state.graph.camera.unproject(point, size) - gesture.anchor;
                    context.state.graph.positions[usize::from(number - 1)] = position;
                }
                GraphDragTarget::Divider => {
                    context.state.graph.split = (point.x / size.x).clamp(0.25, 0.75)
                }
                GraphDragTarget::Pan => {
                    context.state.graph.camera.pan = gesture.pan + point - gesture.anchor;
                    moved_pan |= matches!(input.action, PointerAction::Move { .. });
                }
            }
        }
        if matches!(input.action, PointerAction::Release(PointerButton::Primary)) {
            if !gesture.moved
                && let GraphDragTarget::Node(number) = gesture.target
            {
                context.select_node(number);
            }
        } else {
            context.state.graph.gesture = Some(gesture);
        }
    }
    if moved_pan {
        let displacement = context.state.graph.camera.pan - before_pan;
        context
            .state
            .graph
            .camera
            .sample_pan(displacement, time.delta_secs());
    }
    let dragging = context.state.graph.gesture.is_some();
    context
        .state
        .graph
        .camera
        .advance(time.delta_secs(), dragging);
}
