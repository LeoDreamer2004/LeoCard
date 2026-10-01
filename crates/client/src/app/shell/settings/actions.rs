use super::{SettingsTab, start_table_felt_picker};
use crate::app::runtime::{
    AppearancePreferences, TableAppearance, TableFeltPicker, save_appearance_preferences,
};
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

#[derive(SystemParam)]
struct AppearanceUiResources<'w> {
    table_felt_picker: ResMut<'w, TableFeltPicker>,
    table_appearance: ResMut<'w, TableAppearance>,
}

#[derive(Clone)]
pub(crate) enum SettingsUiAction {
    SelectTab(SettingsTab),
    ToggleMahjongActionVoices,
    ToggleMahjongFanVoices,
    ToggleTexasActionVoices,
    ChooseTableFelt,
    UseDefaultTableFelt,
}

impl DomainUiAction for SettingsUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Settings(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(super) struct SettingsUiActionContext<'w> {
    ui: ResMut<'w, UiState>,
    appearance: ResMut<'w, AppearancePreferences>,
    local: AppearanceUiResources<'w>,
}

pub(super) fn dispatch_settings_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: SettingsUiActionContext,
) {
    dispatch_domain_actions::<SettingsUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<SettingsUiActionContext<'_>> for SettingsUiAction {
    fn handle(&self, context: &mut SettingsUiActionContext<'_>) {
        let ui = &mut *context.ui;
        let appearance = &mut *context.appearance;
        let local = &mut context.local;
        match self {
            SettingsUiAction::SelectTab(tab) => ui.settings.tab = *tab,
            SettingsUiAction::ToggleMahjongActionVoices => {
                appearance.mahjong_action_voices = !appearance.mahjong_action_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ToggleMahjongFanVoices => {
                appearance.mahjong_fan_voices = !appearance.mahjong_fan_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ToggleTexasActionVoices => {
                appearance.texas_action_voices = !appearance.texas_action_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ChooseTableFelt => {
                if local.table_felt_picker.pending.is_none() {
                    match start_table_felt_picker() {
                        Ok(receiver) => {
                            local.table_felt_picker.pending = Some(receiver);
                            local.table_appearance.error = None;
                        }
                        Err(error) => local.table_appearance.error = Some(error),
                    }
                }
            }
            SettingsUiAction::UseDefaultTableFelt => {
                appearance.table_felt_path = None;
                appearance.table_brightness = 1.0;
                local.table_appearance.error = save_appearance_preferences(appearance).err();
            }
        }
    }
}
