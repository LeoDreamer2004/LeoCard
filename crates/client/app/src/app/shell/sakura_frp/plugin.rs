use super::account::cache_account;
use super::actions::{dispatch_sakura_actions, refresh};
use super::input::{
    remember_sakura_scroll, scroll_sakura_lists, submit_sakura_login, sync_sakura_inputs,
};
use super::state::{SakuraState, TaskMessage, TaskOutcome};
use super::view::{advance_startup_modal, sync_account_menu, sync_startup_dialog};
use crate::app::achievements::LocalAchievementTrigger;
use crate::app::runtime::{ClientUpdateSet, PageErrorState};
use crate::app::shell::{ModalAnimationSet, UiActionSet, UiState};
use bevy::prelude::*;
use leocard_achievements::{AchievementTrigger, PersonalEvent};
use leocard_ui::TextInputSet;
use std::mem;
use std::time::{Duration, Instant};

pub(crate) struct SakuraFrpPlugin;

impl Plugin for SakuraFrpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SakuraState>()
            .add_systems(
                Update,
                (dispatch_sakura_actions, submit_sakura_login)
                    .chain()
                    .in_set(UiActionSet),
            )
            .add_systems(Update, scroll_sakura_lists.in_set(ClientUpdateSet::Input))
            .add_systems(Update, sync_account_menu.in_set(ClientUpdateSet::Input))
            .add_systems(
                Update,
                advance_startup_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                (
                    remember_sakura_scroll,
                    poll_sakura_tasks,
                    sync_startup_dialog,
                    poll_sakura_processes,
                    expire_copy_feedback,
                    refresh_periodically,
                )
                    .chain()
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(PostUpdate, sync_sakura_inputs.in_set(TextInputSet::Publish));
    }
}

fn poll_sakura_tasks(
    mut state: ResMut<SakuraState>,
    mut ui: ResMut<UiState>,
    mut error: ResMut<PageErrorState>,
    mut images: ResMut<Assets<Image>>,
    mut achievements: MessageWriter<LocalAchievementTrigger>,
) {
    let messages = {
        let Ok(mut queue) = state.messages.lock() else {
            return;
        };
        mem::take(&mut *queue)
    };
    if messages.is_empty() {
        return;
    }
    let mut rebuild = false;
    for message in messages {
        match message {
            TaskMessage::StartupProgress(progress) => {
                if let Some(startup) = &mut state.startup {
                    startup.stage = progress;
                }
            }
            TaskMessage::AccountRefreshed { token, result } => {
                rebuild = true;
                state.preview_running = false;
                if state.remember_token
                    && state
                        .saved_login
                        .as_ref()
                        .is_some_and(|saved| saved.token == token)
                {
                    match result {
                        Ok(saved) => {
                            error.error = cache_account(&mut state, saved, &mut images).err()
                        }
                        Err(message) if state.open && state.client.is_none() => {
                            error.error = Some(message)
                        }
                        Err(_) => {}
                    }
                }
            }
            TaskMessage::Finished(result) => {
                rebuild = true;
                state.busy = false;
                match *result {
                    Ok(TaskOutcome::LoggedIn(client, dashboard, saved)) => {
                        state.token = saved.token.clone();
                        error.error = cache_account(&mut state, saved, &mut images).err();
                        state.client = Some(client);
                        state.apply_dashboard(dashboard);
                    }
                    Ok(TaskOutcome::Refreshed(dashboard)) => {
                        state.apply_dashboard(dashboard);
                    }
                    Ok(TaskOutcome::Created(dashboard)) => {
                        error.error = None;
                        if let Some(dashboard) = dashboard {
                            state.apply_dashboard(dashboard);
                        }
                        state.name.clear();
                        state.local_port.clear();
                        state.remote.clear();
                    }
                    Ok(TaskOutcome::Deleted(id, failed_to_disconnect, dashboard)) => {
                        state.running.remove(&id);
                        state.tunnels.retain(|tunnel| tunnel.id != id);
                        if let Some(dashboard) = dashboard {
                            state.apply_dashboard(dashboard);
                        }
                        if failed_to_disconnect {
                            error.error = Some(format!("隧道 {id} 已删除，服务端未能立即断开连接"));
                        }
                    }
                    Ok(TaskOutcome::Started(running)) => {
                        let id = running.id;
                        state.running.insert(id, running);
                        achievements.write(LocalAchievementTrigger(AchievementTrigger::Personal(
                            PersonalEvent::SakuraTunnelStarted,
                        )));
                    }
                    Err(message) => {
                        state.startup = None;
                        error.error = Some(message);
                    }
                }
            }
        }
    }
    if rebuild {
        ui.dirty = true;
    }
}

fn poll_sakura_processes(
    mut state: ResMut<SakuraState>,
    mut ui: ResMut<UiState>,
    mut error: ResMut<PageErrorState>,
) {
    let mut exited = Vec::new();
    state.running.retain(|id, process| {
        if process.running().unwrap_or(false) {
            return true;
        }
        exited.push((*id, process.log_path.clone()));
        false
    });
    if let Some((id, log)) = exited.last() {
        if state
            .startup
            .as_ref()
            .is_some_and(|startup| exited.iter().any(|(id, _)| *id == startup.id))
        {
            state.startup = None;
        }
        error.error = Some(format!(
            "隧道 {id} 的 frpc 已退出，请查看 {}",
            log.display()
        ));
        ui.dirty = true;
    }
}

fn refresh_periodically(mut state: ResMut<SakuraState>) {
    if !state.open || state.client.is_none() || state.busy || Instant::now() < state.next_refresh {
        return;
    }
    state.next_refresh = Instant::now() + Duration::from_secs(30);
    refresh(&mut state);
}

fn expire_copy_feedback(mut state: ResMut<SakuraState>, mut ui: ResMut<UiState>) {
    if state
        .copied_address
        .is_some_and(|(_, expires)| Instant::now() >= expires)
    {
        state.copied_address = None;
        ui.dirty = true;
    }
}
