//! 界面缩放。

use super::UiZoom;
use crate::app::presentation::{DESIGN_HEIGHT, DESIGN_WIDTH};
use crate::app::shell::UiState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const MIN_AUTO_SCALE: f32 = 0.5;
const MAX_AUTO_SCALE: f32 = 2.5;
const MIN_MANUAL_ZOOM: f32 = 0.7;
const MAX_MANUAL_ZOOM: f32 = 1.5;

pub(crate) fn update_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut zoom: ResMut<UiZoom>,
    mut scale: ResMut<UiScale>,
    mut ui: ResMut<UiState>,
) {
    let ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if ctrl && (keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd))
    {
        zoom.manual = (zoom.manual + 0.1).min(MAX_MANUAL_ZOOM);
        ui.dirty = true;
    }
    if ctrl
        && (keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract))
    {
        zoom.manual = (zoom.manual - 0.1).max(MIN_MANUAL_ZOOM);
        ui.dirty = true;
    }
    if ctrl && keyboard.just_pressed(KeyCode::Digit0) {
        zoom.manual = 1.0;
        ui.dirty = true;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let effective = calculate_ui_scale(window.width(), window.height(), zoom.manual);
    if (scale.0 - effective).abs() > 0.005 {
        scale.0 = effective;
        ui.dirty = true;
    }
}

pub(crate) fn calculate_ui_scale(width: f32, height: f32, manual_zoom: f32) -> f32 {
    let automatic = (width / DESIGN_WIDTH)
        .min(height / DESIGN_HEIGHT)
        .clamp(MIN_AUTO_SCALE, MAX_AUTO_SCALE);
    automatic * manual_zoom.clamp(MIN_MANUAL_ZOOM, MAX_MANUAL_ZOOM)
}
