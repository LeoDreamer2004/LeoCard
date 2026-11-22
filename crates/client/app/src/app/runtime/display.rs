//! 全屏状态、快捷键和跨平台节能帧率控制。

use super::{AppearancePreferences, ClientUpdateSet};
use crate::app::shell::{UiActionSet, UiState};
use bevy::prelude::*;
#[cfg(not(feature = "web"))]
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};
use bevy::winit::{UpdateMode, WinitSettings};
use leocard_platform::time::Duration;

#[derive(Message)]
pub(crate) struct ToggleFullscreen;

pub(super) struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToggleFullscreen>().add_systems(
            Update,
            (
                apply_fullscreen_requests,
                sync_fullscreen,
                sync_power_saving,
            )
                .chain()
                .after(UiActionSet)
                .in_set(ClientUpdateSet::Sync),
        );
        #[cfg(not(feature = "web"))]
        app.add_systems(
            Update,
            fullscreen_shortcut.before(apply_fullscreen_requests),
        );
    }
}

#[cfg(not(feature = "web"))]
fn fullscreen_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut requests: MessageWriter<ToggleFullscreen>,
) {
    if cfg!(feature = "desktop") && keyboard.just_pressed(KeyCode::F11) {
        requests.write(ToggleFullscreen);
    }
}

fn apply_fullscreen_requests(
    mut requests: MessageReader<ToggleFullscreen>,
    #[cfg(not(feature = "web"))] mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if requests.read().count().is_multiple_of(2) {
        return;
    }
    #[cfg(feature = "web")]
    super::web::toggle_fullscreen();
    #[cfg(not(feature = "web"))]
    if let Ok(mut window) = windows.single_mut() {
        window.mode = if window.mode == WindowMode::Windowed {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
    }
}

fn sync_fullscreen(
    mut preferences: ResMut<AppearancePreferences>,
    mut ui: ResMut<UiState>,
    #[cfg(not(feature = "web"))] windows: Query<&Window, With<PrimaryWindow>>,
) {
    #[cfg(feature = "web")]
    let (supported, fullscreen) = (
        super::web::fullscreen_supported(),
        super::web::is_fullscreen(),
    );
    #[cfg(not(feature = "web"))]
    let (supported, fullscreen) = (
        cfg!(feature = "desktop"),
        windows
            .single()
            .is_ok_and(|window| window.mode != WindowMode::Windowed),
    );
    if preferences.fullscreen_supported != supported || preferences.fullscreen != fullscreen {
        preferences.fullscreen_supported = supported;
        preferences.fullscreen = fullscreen;
        ui.dirty = true;
    }
}

fn sync_power_saving(preferences: Res<AppearancePreferences>, mut settings: ResMut<WinitSettings>) {
    if !preferences.is_changed() {
        return;
    }
    let desired = if preferences.power_saving {
        // Input is still collected normally, but cannot wake extra render frames.
        let mode = UpdateMode::Reactive {
            wait: Duration::from_secs_f64(1.0 / 30.0),
            react_to_device_events: false,
            react_to_user_events: false,
            react_to_window_events: false,
        };
        WinitSettings {
            focused_mode: mode,
            unfocused_mode: mode,
        }
    } else {
        WinitSettings::game()
    };
    if settings.focused_mode != desired.focused_mode
        || settings.unfocused_mode != desired.unfocused_mode
    {
        *settings = desired;
    }
}
