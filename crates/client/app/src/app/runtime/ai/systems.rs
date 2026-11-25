use super::super::{AiPreferences, AppearancePreferences, ClientResource};
use super::state::{AiModels, ModelEvent, ModelStatus};
use super::storage;
use crate::app::shell::{SettingsTab, UiState};
use bevy::prelude::*;
use leocard_ai::{DevicePreference, ModelId};
use leocard_client::ClientPhaseRef;
use leocard_doudizhu::DoudizhuMode;
use leocard_protocol::{ClientCommand, GameRules};
use std::thread;

pub(crate) fn initialize_ai_models(
    mut models: ResMut<AiModels>,
    preferences: Res<AppearancePreferences>,
) {
    for id in ModelId::ALL {
        models.states.insert(id, ModelStatus::Loading);
    }
    let sender = models.sender.clone();
    thread::spawn(move || {
        for id in ModelId::ALL {
            let status = match storage::load(id) {
                Ok(true) => ModelStatus::Ready,
                Ok(false) => ModelStatus::Missing,
                Err(e) => ModelStatus::Failed(e),
            };
            let _ = sender.send(ModelEvent::Status(id, status));
        }
    });
    if !preferences.ai.enabled.is_empty() && preferences.ai.device != DevicePreference::Cpu {
        models.probe_device();
    }
}

pub(crate) fn poll_ai_models(mut models: ResMut<AiModels>, mut ui: ResMut<UiState>) {
    let events: Vec<_> = models
        .receiver
        .lock()
        .expect("AI event queue")
        .try_iter()
        .collect();
    let changed = !events.is_empty();
    for event in events {
        match event {
            ModelEvent::Status(id, status) => {
                models.states.insert(id, status);
            }
            ModelEvent::Device(result) => {
                models.probing_gpu = false;
                models.gpu = Some(result);
            }
        }
    }
    if changed && ui.settings.open && ui.settings.tab == SettingsTab::Ai {
        ui.dirty = true;
    }
}

pub(crate) fn ai_model_for_rules(rules: &GameRules) -> Option<ModelId> {
    match rules {
        GameRules::Doudizhu(rules) if rules.mode == DoudizhuMode::Three => Some(ModelId::Douzero),
        GameRules::Mahjong(_) => Some(ModelId::Kdens3),
        GameRules::Guandan(_) => Some(ModelId::GuandanActor),
        _ => None,
    }
}

impl AiModels {
    pub(crate) fn configured_model(
        &self,
        preferences: &AiPreferences,
        rules: &GameRules,
    ) -> Option<ModelId> {
        ai_model_for_rules(rules).filter(|id| {
            preferences.enabled.contains(id)
                && self.status(*id).ready()
                && (preferences.device != DevicePreference::Gpu || self.gpu_available())
        })
    }
}

pub(crate) fn sync_lobby_ai(
    models: Res<AiModels>,
    preferences: Res<AppearancePreferences>,
    client: Option<ResMut<ClientResource>>,
    mut previous: Local<Option<(u64, Option<ModelId>, DevicePreference)>>,
) {
    let Some(mut client) = client else {
        *previous = None;
        return;
    };
    if !client.0.model().is_host() || !matches!(client.0.model().phase(), ClientPhaseRef::Lobby(_))
    {
        *previous = None;
        return;
    }
    let Some(rules) = client.0.model().game_rules() else {
        return;
    };
    let model = models.configured_model(&preferences.ai, rules);
    let stamp = (client.0.session_id(), model, preferences.ai.device);
    if previous.as_ref() != Some(&stamp)
        && client.0.send(ClientCommand::ConfigureBotAi {
            model,
            device: preferences.ai.device,
        })
    {
        *previous = Some(stamp);
    }
}
