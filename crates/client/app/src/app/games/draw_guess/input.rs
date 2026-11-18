use super::actions::send_draw;
use super::audio::DrawingAudio;
use super::canvas::CanvasRenderer;
use super::client::DrawGuessClient;
use super::state::{
    CanvasPaper, DrawGuessUiState, DrawTool, GuessHistory, GuessInput, can_draw, can_message,
};
use crate::app::runtime::ClientResource;
use crate::app::shell::{ConfirmationDialog, PageMotion, UiState, game_command};
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::window::PrimaryWindow;
use leocard_draw_guess::{CANVAS_HEIGHT, CANVAS_WIDTH, CanvasPoint, DrawAction};
use leocard_protocol::{DrawGuessCommand, GameSnapshot};
use leocard_ui::{TextInputAction, TextInputEvent};

#[derive(SystemParam)]
pub(super) struct DrawingInput<'w, 's> {
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    time: Res<'w, Time>,
    focus: Res<'w, InputFocus>,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
    papers: Query<
        'w,
        's,
        (
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static PickingInteraction,
        ),
        With<CanvasPaper>,
    >,
    history: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<GuessHistory>>,
    editors: Query<'w, 's, Entity, With<EditableText>>,
    shell: Res<'w, UiState>,
    confirmation: Res<'w, ConfirmationDialog>,
    motion: Res<'w, PageMotion>,
}

impl DrawingInput<'_, '_> {
    fn blocked(&self) -> bool {
        !self.windows.single().is_ok_and(|window| window.focused)
            || self.shell.settings.open
            || self.shell.shop.open
            || self.shell.profile.open
            || self.shell.leaving_room
            || self.confirmation.is_open()
            || self.motion.active()
    }

    fn pointer(&self) -> Option<(CanvasPoint, bool)> {
        let window = self.windows.single().ok()?;
        let cursor = window.cursor_position()? * window.scale_factor();
        if self.history.iter().any(|(node, transform)| {
            normalized(cursor, node, transform)
                .is_some_and(|point| point.cmpge(Vec2::ZERO).all() && point.cmple(Vec2::ONE).all())
        }) {
            return None;
        }
        let (node, transform, interaction) = self.papers.single().ok()?;
        let point = normalized(cursor, node, transform)?;
        let inside = point.cmpge(Vec2::ZERO).all()
            && point.cmple(Vec2::ONE).all()
            && *interaction != PickingInteraction::None;
        Some((
            CanvasPoint {
                x: (point.x * f32::from(CANVAS_WIDTH)).clamp(0.0, f32::from(CANVAS_WIDTH - 1))
                    as u16,
                y: (point.y * f32::from(CANVAS_HEIGHT)).clamp(0.0, f32::from(CANVAS_HEIGHT - 1))
                    as u16,
            },
            inside,
        ))
    }
}

pub(super) fn normalized(
    cursor: Vec2,
    node: &ComputedNode,
    transform: &UiGlobalTransform,
) -> Option<Vec2> {
    if node.size().min_element() <= 0.0 {
        return None;
    }
    Some(transform.try_inverse()?.transform_point2(cursor) / node.size() + Vec2::splat(0.5))
}

pub(super) fn draw_input(
    input: DrawingInput,
    mut ui: ResMut<DrawGuessUiState>,
    mut prediction: ResMut<DrawGuessClient>,
    mut audio: ResMut<DrawingAudio>,
    renderer: Res<CanvasRenderer>,
    mut client: Option<ResMut<ClientResource>>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(game) = client
        .0
        .model()
        .game_snapshot()
        .and_then(GameSnapshot::draw_guess)
    else {
        return;
    };
    ui.reconcile(game);
    let turn = game.state.turn;
    if !can_draw(game) {
        return;
    }
    if input.blocked() || client.0.reconnecting_message().is_some() {
        finish_stroke(client, &mut prediction, &mut ui, turn);
        return;
    }
    let control = input.keyboard.pressed(KeyCode::ControlLeft)
        || input.keyboard.pressed(KeyCode::ControlRight);
    let editing = input
        .focus
        .get()
        .is_some_and(|entity| input.editors.contains(entity));
    if control && !editing {
        let action = if input.keyboard.just_pressed(KeyCode::KeyZ) {
            Some(DrawAction::Undo)
        } else if input.keyboard.just_pressed(KeyCode::KeyR) {
            Some(DrawAction::Redo)
        } else {
            None
        };
        if let Some(action) = action {
            finish_stroke(client, &mut prediction, &mut ui, turn);
            send_draw(client, &mut prediction, turn, action);
            return;
        }
    }
    let pointer = input.pointer();
    if input.mouse.just_pressed(MouseButton::Left)
        && let Some((point, true)) = pointer
    {
        match ui.tool {
            DrawTool::Pen | DrawTool::AreaEraser => {
                let action = DrawAction::Begin {
                    point,
                    color: ui.color,
                    size: ui.size,
                    eraser: ui.tool == DrawTool::AreaEraser,
                };
                ui.drawing = send_draw(client, &mut prediction, turn, action);
                ui.last_point = Some(point);
                if ui.drawing {
                    audio.stroke(ui.tool);
                }
            }
            DrawTool::Fill => {
                if send_draw(
                    client,
                    &mut prediction,
                    turn,
                    DrawAction::Fill {
                        point,
                        color: ui.color,
                    },
                ) {
                    audio.fill();
                }
            }
            DrawTool::StrokeEraser => {}
        }
    }
    if input.mouse.pressed(MouseButton::Left)
        && ui.tool == DrawTool::StrokeEraser
        && let Some((point, true)) = pointer
        && (input.mouse.just_pressed(MouseButton::Left) || ui.last_point != Some(point))
        && let Some(id) = renderer
            .raster
            .as_ref()
            .and_then(|raster| raster.stroke_at(point))
    {
        ui.last_point = Some(point);
        if send_draw(
            client,
            &mut prediction,
            turn,
            DrawAction::EraseStroke { id },
        ) {
            audio.stroke(DrawTool::StrokeEraser);
        }
    }
    if ui.drawing {
        if !input.mouse.pressed(MouseButton::Left) {
            finish_stroke(client, &mut prediction, &mut ui, turn);
        } else {
            if let Some((point, inside)) = pointer
                && ui.last_point != Some(point)
            {
                ui.queued_points.push(point);
                ui.last_point = Some(point);
                if inside {
                    audio.stroke(ui.tool);
                }
            }
            ui.send_elapsed += input.time.delta_secs();
            if ui.send_elapsed >= 1.0 / 30.0 || ui.queued_points.len() >= 64 {
                flush_points(client, &mut prediction, &mut ui, turn);
            }
        }
    }
    if !input.mouse.pressed(MouseButton::Left) {
        ui.last_point = None;
    }
}

fn flush_points(
    client: &mut ClientResource,
    prediction: &mut DrawGuessClient,
    ui: &mut DrawGuessUiState,
    turn: u32,
) {
    ui.send_elapsed = 0.0;
    while !ui.queued_points.is_empty() {
        let count = ui.queued_points.len().min(64);
        let points = ui.queued_points.drain(..count).collect();
        if !send_draw(client, prediction, turn, DrawAction::Extend { points }) {
            ui.queued_points.clear();
            break;
        }
    }
}

fn finish_stroke(
    client: &mut ClientResource,
    prediction: &mut DrawGuessClient,
    ui: &mut DrawGuessUiState,
    turn: u32,
) {
    if !ui.drawing {
        return;
    }
    flush_points(client, prediction, ui, turn);
    send_draw(client, prediction, turn, DrawAction::End);
    ui.drawing = false;
    ui.last_point = None;
}

pub(super) fn guess_input(
    mut events: MessageReader<TextInputEvent>,
    mut inputs: Query<(Entity, &mut EditableText), With<GuessInput>>,
    mut ui: ResMut<DrawGuessUiState>,
    mut client: Option<ResMut<ClientResource>>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(game) = client
        .0
        .model()
        .game_snapshot()
        .and_then(GameSnapshot::draw_guess)
    else {
        return;
    };
    let before = ui.key;
    ui.reconcile(game);
    if before != ui.key {
        for (_, mut input) in &mut inputs {
            input.clear();
        }
    }
    let latest = game
        .state
        .history
        .iter()
        .filter(|message| message.player == game.you)
        .map(|message| message.serial)
        .max()
        .unwrap_or(0);
    if latest > ui.last_guess_serial {
        if let Some(submitted) = ui.submitted.take() {
            for (_, mut input) in &mut inputs {
                if input.value().to_string() == submitted {
                    input.clear();
                }
            }
        }
        ui.last_guess_serial = latest;
    }
    for (_, input) in &inputs {
        ui.input = input.value().to_string();
    }
    let turn = game.state.turn;
    let allowed = can_message(game);
    for event in events.read() {
        if event.action == TextInputAction::Submit
            && allowed
            && inputs.contains(event.entity)
            && !ui.input.trim().is_empty()
        {
            client.0.send(game_command(DrawGuessCommand::Guess {
                turn,
                text: ui.input.clone(),
            }));
            ui.submitted = Some(ui.input.clone());
        }
    }
}
