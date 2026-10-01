//! 文本输入法与界面缩放。

use super::super::{ChatPanelState, UiState};
use super::{InputField, UiZoom};
use crate::app::presentation::{DESIGN_HEIGHT, DESIGN_WIDTH};
use crate::app::runtime::{ClientResource, ConnectionDraft};
use crate::app::shell::DeveloperHandInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_client::ClientPhaseRef;

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

/// Winit only forwards composition and commit messages from a system input
/// method while IME support is enabled on the window. Keep it active for the
/// player-name field and chat, but disable it for numeric/address/developer
/// inputs so those fields continue to receive exact key presses.
pub(crate) fn sync_ime_enabled(
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionDraft>,
    chat: Res<ChatPanelState>,
    developer_hand: Res<DeveloperHandInput>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let connection_visible = client.as_deref().is_none_or(|client| {
        matches!(
            client.0.model().phase(),
            ClientPhaseRef::Idle | ClientPhaseRef::Closed
        )
    });
    let enabled = !developer_hand.focused
        && (chat.focused || (connection_visible && form.active == InputField::PlayerName));
    if window.ime_enabled != enabled {
        window.ime_enabled = enabled;
    }
    if enabled {
        window.ime_position = if chat.focused {
            Vec2::new(
                (window.width() - 310.0).max(0.0),
                (window.height() - 54.0).max(0.0),
            )
        } else {
            Vec2::new(window.width() * 0.5, window.height() * 0.24)
        };
    }
}
