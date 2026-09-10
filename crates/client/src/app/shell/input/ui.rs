//! Local input, scaling, card selection, and OS-backed image picking.

use super::super::{ChatPanelState, UiState};
use super::{InputField, UiZoom};
use crate::app::games::{HandCardSlot, MahjongHandTile, ShengjiHandCardSlot, UnoHandCardButton};
use crate::app::presentation::{BackgroundButtonTint, ButtonTint, DESIGN_HEIGHT, DESIGN_WIDTH};
use crate::app::runtime::{ClientResource, ConnectionDraft, UiAssets};
use crate::app::shell::DeveloperHandInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leocard_client::ClientPhaseRef;
use std::collections::HashSet;

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

#[expect(
    clippy::type_complexity,
    reason = "separate filtered Bevy queries update image and background buttons"
)]
pub(crate) fn update_button_tints(
    mut image_buttons: Query<
        (&Interaction, &ButtonTint, &mut ImageNode),
        (Changed<Interaction>, Without<BackgroundButtonTint>),
    >,
    mut background_buttons: Query<
        (&Interaction, &ButtonTint, &mut BackgroundColor),
        (Changed<Interaction>, With<BackgroundButtonTint>),
    >,
) {
    for (interaction, tint, mut image) in &mut image_buttons {
        image.color = match interaction {
            Interaction::None => tint.normal,
            Interaction::Hovered => tint.hovered,
            Interaction::Pressed => tint.pressed,
        };
    }
    for (interaction, tint, mut background) in &mut background_buttons {
        background.0 = match interaction {
            Interaction::None => tint.normal,
            Interaction::Hovered => tint.hovered,
            Interaction::Pressed => tint.pressed,
        };
    }
}

#[expect(
    clippy::type_complexity,
    reason = "the Bevy query precisely filters newly pressed ordinary buttons"
)]
pub(crate) fn play_button_click_sounds(
    buttons: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<Button>,
            Without<HandCardSlot>,
            Without<UnoHandCardButton>,
        ),
    >,
    assets: Res<UiAssets>,
    mut commands: Commands,
) {
    if assets.audio.button_click_sounds.is_empty() {
        return;
    }
    for interaction in &buttons {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }
        let sound = assets.audio.button_click_sounds
            [fastrand::usize(..assets.audio.button_click_sounds.len())]
        .clone();
        commands.spawn((AudioPlayer::new(sound), PlaybackSettings::DESPAWN));
    }
}

#[expect(
    clippy::type_complexity,
    reason = "disjoint filtered Bevy queries separate changed and animated buttons"
)]
pub(crate) fn animate_button_presses(
    time: Res<Time>,
    changed: Query<
        (Entity, &Interaction),
        (
            Changed<Interaction>,
            With<Button>,
            Without<HandCardSlot>,
            Without<ShengjiHandCardSlot>,
            Without<MahjongHandTile>,
        ),
    >,
    mut buttons: Query<
        (&Interaction, &mut UiTransform),
        (
            With<Button>,
            Without<HandCardSlot>,
            Without<ShengjiHandCardSlot>,
            Without<MahjongHandTile>,
        ),
    >,
    mut active: Local<HashSet<Entity>>,
) {
    active.extend(changed.iter().map(|(entity, _)| entity));
    if active.is_empty() {
        return;
    }
    let response = 1.0 - (-time.delta_secs() * 28.0).exp();
    let entities = active.iter().copied().collect::<Vec<_>>();
    for entity in entities {
        let Ok((interaction, mut transform)) = buttons.get_mut(entity) else {
            active.remove(&entity);
            continue;
        };
        let (target_scale, target_y) = match interaction {
            Interaction::Pressed => (0.97, 1.5),
            Interaction::Hovered => (1.015, 0.0),
            Interaction::None => (1.0, 0.0),
        };
        let scale = transform.scale.x + (target_scale - transform.scale.x) * response;
        let current_y = match transform.translation.y {
            Val::Px(value) => value,
            _ => 0.0,
        };
        let y = current_y + (target_y - current_y) * response;
        if (scale - target_scale).abs() < 0.000_5 && (y - target_y).abs() < 0.01 {
            if transform.scale.x != target_scale || current_y != target_y {
                transform.scale = Vec2::splat(target_scale);
                transform.translation.y = px(target_y);
            }
            active.remove(&entity);
        } else {
            transform.scale = Vec2::splat(scale);
            transform.translation.y = px(y);
        }
    }
}
