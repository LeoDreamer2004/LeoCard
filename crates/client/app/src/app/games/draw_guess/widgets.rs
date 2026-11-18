use super::assets::DrawGuessAssets;
use super::client::DrawGuessClient;
use super::state::{DrawGuessUiState, DrawTool, can_message};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, MUTED, TEXT, add_text};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::UiAction;
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;
use leocard_draw_guess::Ink;
use leocard_protocol::GameSnapshot;
use leocard_ui::{UiComposer, spawn_node};

#[derive(Clone, Copy)]
pub(super) enum ButtonKind {
    Plain,
    Tool(DrawTool),
    Color(Ink),
    Size(u8),
    Undo,
    Redo,
    Submit,
}

#[derive(Component)]
pub(super) struct DrawingButton {
    kind: ButtonKind,
    label: Entity,
    enabled: bool,
}

pub(super) struct ButtonSpec<'a> {
    pub label: &'a str,
    pub action: Option<UiAction>,
    pub width: Val,
    pub height: f32,
    pub kind: ButtonKind,
    pub icon: Option<Handle<Image>>,
}

impl<'a> ButtonSpec<'a> {
    pub fn new(label: &'a str, action: Option<UiAction>, width: Val) -> Self {
        Self {
            label,
            action,
            width,
            height: 34.0,
            kind: ButtonKind::Plain,
            icon: None,
        }
    }
}

pub(super) fn button(
    commands: &mut UiComposer,
    parent: Entity,
    spec: ButtonSpec<'_>,
    assets: &UiAssets,
    art: &DrawGuessAssets,
) -> Entity {
    let enabled = spec.action.is_some();
    let scale = GAME_SIDE_CONTROL_SCALE;
    let button = spawn_node(
        commands,
        parent,
        Node {
            width: match spec.width {
                Val::Px(width) => px(width * scale),
                other => other,
            },
            height: px(spec.height * scale),
            min_width: px(0),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: if spec.icon.is_some() {
                FlexDirection::Column
            } else {
                FlexDirection::Row
            },
            row_gap: px(1),
            padding: UiRect::horizontal(px(2.0 * scale)),
            ..default()
        },
        None,
    );
    commands
        .entity(button)
        .insert((Button, art.controls.button(enabled, false)));
    if let Some(action) = spec.action {
        commands.entity(button).insert(action);
    } else {
        commands.entity(button).insert(InteractionDisabled);
    }
    let font_size = if spec.icon.is_some() {
        12.0
    } else if spec.height <= 27.0 {
        14.0
    } else {
        16.0
    } * scale;
    if let Some(icon) = spec.icon {
        commands.child(
            button,
            (
                Node {
                    width: px(20.0 * scale),
                    height: px(20.0 * scale),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::new(icon),
                Pickable::IGNORE,
            ),
        );
    }
    let label = add_text(
        commands,
        button,
        spec.label,
        font_size,
        if enabled { TEXT } else { MUTED },
        assets,
    );
    commands
        .entity(label)
        .insert((Pickable::IGNORE, TextLayout::no_wrap()));
    commands.entity(button).insert(DrawingButton {
        kind: spec.kind,
        label,
        enabled,
    });
    button
}

#[derive(SystemParam)]
pub(super) struct ButtonSkinContext<'w> {
    ui: Res<'w, DrawGuessUiState>,
    prediction: Res<'w, DrawGuessClient>,
    art: Res<'w, DrawGuessAssets>,
    focus: Res<'w, InputFocus>,
    client: Option<Res<'w, ClientResource>>,
}

pub(super) fn sync_buttons(
    mut commands: Commands,
    context: ButtonSkinContext,
    mut buttons: Query<(Entity, &DrawingButton, &PickingInteraction, &mut ImageNode)>,
    mut labels: Query<&mut TextColor>,
) {
    if buttons.is_empty() {
        return;
    }
    let ui = &context.ui;
    let document = context.prediction.predicted();
    let can_message = context
        .client
        .as_deref()
        .and_then(|client| client.0.model().game_snapshot())
        .and_then(GameSnapshot::draw_guess)
        .is_some_and(can_message);
    for (entity, button, interaction, mut image) in &mut buttons {
        let (enabled, selected) = match button.kind {
            ButtonKind::Plain => (button.enabled, false),
            ButtonKind::Tool(tool) => (true, ui.tool == tool),
            ButtonKind::Color(color) => (true, ui.color == color),
            ButtonKind::Size(size) => (true, ui.size == size),
            ButtonKind::Undo => (document.can_undo(), false),
            ButtonKind::Redo => (document.can_redo(), false),
            ButtonKind::Submit => (can_message && !ui.input.trim().is_empty(), false),
        };
        if enabled {
            commands.entity(entity).remove::<InteractionDisabled>();
        } else {
            commands.entity(entity).insert(InteractionDisabled);
        }
        let highlighted = selected
            || (enabled
                && (*interaction != PickingInteraction::None
                    || context.focus.get() == Some(entity)));
        let next = context.art.controls.button(enabled, highlighted);
        if image.image != next.image || image.color != next.color {
            *image = next;
        }
        if let Ok(mut color) = labels.get_mut(button.label) {
            let next = if !enabled {
                MUTED
            } else if highlighted {
                Color::srgb(0.08, 0.10, 0.16)
            } else {
                TEXT
            };
            if color.0 != next {
                color.0 = next;
            }
        }
    }
}
