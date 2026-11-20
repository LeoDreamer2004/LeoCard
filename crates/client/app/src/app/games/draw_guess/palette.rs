use super::input::normalized;
use super::state::DrawGuessUiState;
use crate::app::presentation::game_side_control_scale;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui::{BackgroundGradient, ColorStop, Gradient, LinearGradient};
use bevy::window::PrimaryWindow;
use leocard_draw_guess::Ink;
use leocard_platform::is_mobile;
use leocard_ui::{TouchDragTarget, UiComposer, spawn_node};

#[derive(Component, Clone, Copy)]
pub(super) enum ColorPicker {
    Square,
    Hue,
}

#[derive(Component)]
pub(super) struct InkSwatch;

#[derive(Component)]
pub(super) struct PickerHandle(ColorPicker);

pub(super) fn render(commands: &mut UiComposer, panel: Entity, ui: &DrawGuessUiState) {
    let swatch = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(12.0 * game_side_control_scale()),
            flex_shrink: 0.0,
            ..default()
        },
        Some(ink_color(ui.color)),
    );
    commands.entity(swatch).insert(InkSwatch);
    let square = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(if is_mobile() { 90.0 } else { 76.0 }),
            flex_shrink: 0.0,
            ..default()
        },
        Some(Color::srgb(1.0, 0.0, 0.0)),
    );
    commands.entity(square).insert((
        ColorPicker::Square,
        TouchDragTarget,
        Pickable::default(),
        PickingInteraction::default(),
    ));
    for gradient in [
        LinearGradient::to_right(vec![
            ColorStop::percent(Color::WHITE, 0.0),
            ColorStop::percent(Color::WHITE.with_alpha(0.0), 100.0),
        ]),
        LinearGradient::to_bottom(vec![
            ColorStop::percent(Color::BLACK.with_alpha(0.0), 0.0),
            ColorStop::percent(Color::BLACK, 100.0),
        ]),
    ] {
        commands.child(
            square,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(0),
                    ..default()
                },
                BackgroundGradient(vec![Gradient::Linear(gradient)]),
                Pickable::IGNORE,
            ),
        );
    }
    let hue = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            height: px(20.0 * game_side_control_scale()),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    let colors = [
        Color::srgb(1.0, 0.0, 0.0),
        Color::srgb(1.0, 1.0, 0.0),
        Color::srgb(0.0, 1.0, 0.0),
        Color::srgb(0.0, 1.0, 1.0),
        Color::srgb(0.0, 0.0, 1.0),
        Color::srgb(1.0, 0.0, 1.0),
        Color::srgb(1.0, 0.0, 0.0),
    ];
    commands.entity(hue).insert((
        ColorPicker::Hue,
        TouchDragTarget,
        Pickable::default(),
        PickingInteraction::default(),
        BackgroundColor(Color::WHITE),
        BackgroundGradient(vec![Gradient::Linear(LinearGradient::to_right(
            colors
                .into_iter()
                .enumerate()
                .map(|(index, color)| ColorStop::percent(color, index as f32 / 6.0 * 100.0))
                .collect(),
        ))]),
    ));
    for (parent, kind) in [(square, ColorPicker::Square), (hue, ColorPicker::Hue)] {
        commands.child(
            parent,
            (
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(0),
                    top: percent(0),
                    width: px(8),
                    height: px(8),
                    border: UiRect::all(px(2)),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                Outline {
                    width: px(1),
                    offset: px(0),
                    color: Color::BLACK,
                },
                PickerHandle(kind),
                UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
                Pickable::IGNORE,
            ),
        );
    }
}

pub(super) fn ink_color(ink: Ink) -> Color {
    Color::srgb_u8(ink.0[0], ink.0[1], ink.0[2])
}

pub(super) fn update_picker(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui: ResMut<DrawGuessUiState>,
    pickers: Query<(
        Entity,
        &ColorPicker,
        &ComputedNode,
        &UiGlobalTransform,
        &PickingInteraction,
    )>,
) {
    if !mouse.pressed(MouseButton::Left) {
        ui.picker_drag = None;
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    if !window.focused {
        ui.picker_drag = None;
        return;
    }
    let Some(cursor) = window
        .cursor_position()
        .map(|position| position * window.scale_factor())
    else {
        return;
    };
    for (entity, kind, node, transform, interaction) in &pickers {
        if mouse.just_pressed(MouseButton::Left) && *interaction != PickingInteraction::None {
            ui.picker_drag = Some(entity);
        }
        if ui.picker_drag == Some(entity)
            && let Some(position) = normalized(cursor, node, transform)
        {
            match kind {
                ColorPicker::Square => {
                    ui.saturation = position.x.clamp(0.0, 1.0);
                    ui.value = 1.0 - position.y.clamp(0.0, 1.0);
                }
                ColorPicker::Hue => ui.hue = position.x.clamp(0.0, 0.99999),
            }
            ui.set_hsv();
        }
    }
}

pub(super) fn sync_palette(
    ui: Res<DrawGuessUiState>,
    mut pickers: Query<(&ColorPicker, &mut BackgroundColor)>,
    mut swatches: Query<&mut BackgroundColor, (With<InkSwatch>, Without<ColorPicker>)>,
    mut handles: Query<(&PickerHandle, &mut Node)>,
) {
    let mut hue = DrawGuessUiState {
        hue: ui.hue,
        saturation: 1.0,
        value: 1.0,
        ..default()
    };
    hue.set_hsv();
    for (kind, mut background) in &mut pickers {
        if matches!(kind, ColorPicker::Square) {
            background.0 = ink_color(hue.color);
        }
    }
    for mut swatch in &mut swatches {
        swatch.0 = ink_color(ui.color);
    }
    for (handle, mut node) in &mut handles {
        match handle.0 {
            ColorPicker::Square => {
                node.left = percent(ui.saturation * 100.0);
                node.top = percent((1.0 - ui.value) * 100.0);
            }
            ColorPicker::Hue => {
                node.left = percent(ui.hue * 100.0);
                node.top = percent(50);
                node.width = px(4);
                node.height = percent(100);
            }
        }
    }
}
