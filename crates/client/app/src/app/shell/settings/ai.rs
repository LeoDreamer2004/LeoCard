use super::SettingsTab;
use crate::app::runtime::{AppearancePreferences, save_appearance_preferences};
use crate::app::shell::UiState;
use bevy::prelude::*;
use bevy::text::EditableText;

#[derive(Component)]
pub(super) struct AiProxyPortInput;

pub(super) fn sync_ai_settings_inputs(
    inputs: Query<&EditableText, With<AiProxyPortInput>>,
    mut preferences: ResMut<AppearancePreferences>,
    mut ui: ResMut<UiState>,
) {
    if !ui.settings.open || ui.settings.tab != SettingsTab::Ai {
        if ui.settings.ai_device_menu_open {
            ui.settings.ai_device_menu_open = false;
        }
        return;
    }
    for input in &inputs {
        if input.value() != preferences.ai.download_proxy_port.as_str() {
            preferences.ai.download_proxy_port = input.value().to_string();
            if let Err(error) = save_appearance_preferences(&preferences) {
                warn!("{error}");
            }
        }
    }
}
