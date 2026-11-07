use super::{SettingsTab, start_table_felt_picker};
use crate::app::achievements::LocalAchievementTrigger;
use crate::app::runtime::{
    AppearancePreferences, ClientResource, GlobalPreferences, TableAppearance, TableFeltPicker,
    TableSurfaceKind, save_appearance_preferences,
};
use crate::app::shell::{
    ConfirmationDialog, CozyButtonVariant, DomainUiAction, PressedUiAction, UiAction,
    UiActionHandler, UiState, dispatch_domain_actions,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_game_common::PlayerId;
use leocard_protocol::ClientCommand;

#[derive(SystemParam)]
struct AppearanceUiResources<'w> {
    table_felt_picker: ResMut<'w, TableFeltPicker>,
    table_appearance: ResMut<'w, TableAppearance>,
}

#[derive(Clone)]
pub(crate) enum SettingsUiAction {
    SelectTab(SettingsTab),
    ToggleSpectatorInteractions,
    RequestKickPlayer(PlayerId),
    KickPlayer(PlayerId),
    #[cfg(not(target_os = "android"))]
    ToggleTurnNotifications,
    ToggleMahjongActionVoices,
    ToggleMahjongFanVoices,
    ToggleMahjongTileVoices,
    ToggleTexasActionVoices,
    ToggleQiGuiActionVoices,
    ToggleDoudizhuActionVoices,
    ToggleGuandanActionVoices,
    ToggleMcGuessBackground,
    ChooseTableFelt(TableSurfaceKind),
    UseDefaultTableFelt(TableSurfaceKind),
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
    achievements: MessageWriter<'w, LocalAchievementTrigger>,
    client: Option<ResMut<'w, ClientResource>>,
    confirmation: ResMut<'w, ConfirmationDialog>,
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
            SettingsUiAction::ToggleSpectatorInteractions => {
                if let Some(client) = context.client.as_deref_mut()
                    && client.0.model().is_host()
                    && let Some(room) = client.0.model().room_moderation()
                {
                    let blocked = !room.spectator_interactions_blocked;
                    client
                        .0
                        .send(ClientCommand::SetSpectatorInteractionsBlocked { blocked });
                }
            }
            SettingsUiAction::RequestKickPlayer(player) => {
                if let Some(client) = context.client.as_deref()
                    && client.0.model().is_host()
                    && let Some(room) = client.0.model().room_moderation()
                    && let Some(target) = room.participants.iter().find(|target| {
                        target.id == *player && !target.is_bot && Some(target.id) != room.host
                    })
                {
                    context.confirmation.show_with_button(
                        "踢出玩家".to_owned(),
                        format!("确定将 {} 踢出房间吗？", target.name),
                        UiAction::Settings(Self::KickPlayer(*player)),
                        "踢出",
                        CozyButtonVariant::Danger,
                    );
                }
            }
            SettingsUiAction::KickPlayer(player) => {
                if let Some(client) = context.client.as_deref_mut()
                    && client.0.model().is_host()
                {
                    client.0.send(ClientCommand::KickPlayer { player: *player });
                }
            }
            #[cfg(not(target_os = "android"))]
            SettingsUiAction::ToggleTurnNotifications => {
                appearance.turn_notifications = !appearance.turn_notifications;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
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
            SettingsUiAction::ToggleMahjongTileVoices => {
                appearance.mahjong_tile_voices = !appearance.mahjong_tile_voices;
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
            SettingsUiAction::ToggleQiGuiActionVoices => {
                appearance.qigui_action_voices = !appearance.qigui_action_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ToggleDoudizhuActionVoices => {
                appearance.doudizhu_action_voices = !appearance.doudizhu_action_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ToggleGuandanActionVoices => {
                appearance.guandan_action_voices = !appearance.guandan_action_voices;
                if let Err(error) = save_appearance_preferences(appearance) {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ToggleMcGuessBackground => {
                appearance.games.mc_guess_random_background =
                    !appearance.games.mc_guess_random_background;
                if let Err(error) = appearance.games.save() {
                    warn!("{error}");
                }
            }
            SettingsUiAction::ChooseTableFelt(kind) => {
                if local.table_felt_picker.pending.is_none() {
                    match start_table_felt_picker() {
                        Ok(receiver) => {
                            local.table_felt_picker.pending = Some((*kind, receiver));
                            local.table_appearance.error = None;
                        }
                        Err(error) => local.table_appearance.error = Some(error),
                    }
                }
            }
            SettingsUiAction::UseDefaultTableFelt(kind) => {
                let changed = appearance.felt_path_mut(*kind).take().is_some();
                let defaults = GlobalPreferences::default();
                let brightness = match kind {
                    TableSurfaceKind::Cards => defaults.table_brightness,
                    TableSurfaceKind::Mahjong => defaults.mahjong_table_brightness,
                };
                appearance.set_brightness(*kind, brightness);
                local.table_appearance.error = save_appearance_preferences(appearance).err();
                if changed && local.table_appearance.error.is_none() {
                    context.achievements.write(LocalAchievementTrigger(
                        AchievementTrigger::Personal(PersonalEvent::TableBackgroundChanged),
                    ));
                }
            }
        }
    }
}
