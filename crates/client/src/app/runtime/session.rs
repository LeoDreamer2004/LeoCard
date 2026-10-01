//! 网络快照轮询与客户端状态同步。

use super::{HostRulePreferences, PageErrorState, save_host_rule_preferences};
use crate::app::shell::NetworkUiContext;
use bevy::prelude::*;
use leocard_client::{ClientModel, LocalPlayerProfile, TcpGameClient};
use leocard_protocol::{PlayerId, ServerEvent};

/// Accepted transient server events; independent readers may consume them.
#[derive(Message)]
pub(crate) struct ServerNotification {
    pub player: Option<PlayerId>,
    pub event: ServerEvent,
}

#[derive(Resource)]
pub(crate) struct ClientResource(pub TcpGameClient);

pub(super) fn poll_network(
    mut client: Option<ResMut<ClientResource>>,
    mut host_rules: ResMut<HostRulePreferences>,
    mut page_error: ResMut<PageErrorState>,
    mut profile: ResMut<LocalPlayerProfile>,
    mut ui: NetworkUiContext,
    mut notifications: MessageWriter<ServerNotification>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let before = ui.before_poll(&client.0);
    if !client.0.poll_with_events(|player, event| {
        if !matches!(
            event,
            ServerEvent::GameSnapshot(_)
                | ServerEvent::LobbySnapshot(_)
                | ServerEvent::AvatarData { .. }
                | ServerEvent::Heartbeat
        ) {
            notifications.write(ServerNotification {
                player,
                event: event.clone(),
            });
        }
    }) {
        return;
    }
    sync_local_profile(client.0.model(), &mut profile, &mut page_error);
    sync_host_rule_preferences(client.0.model(), &mut host_rules, &mut page_error);
    ui.after_poll(&client.0, before, &mut page_error);
}

fn sync_local_profile(
    model: &ClientModel,
    profile: &mut LocalPlayerProfile,
    page_error: &mut PageErrorState,
) {
    let mut changed = model
        .last_finished_match()
        .is_some_and(|(match_id, changes)| profile.apply_finished_match(match_id, changes));
    if let Some(profiles) = model.local_game_profiles() {
        changed |= profile.sync_game_profiles(profiles);
    }
    if changed && let Err(error) = profile.save() {
        page_error.error = Some(error);
    }
}

fn sync_host_rule_preferences(
    model: &ClientModel,
    host_rules: &mut HostRulePreferences,
    page_error: &mut PageErrorState,
) {
    let accepted_rules = model
        .lobby()
        .zip(model.you())
        .filter(|(lobby, you)| lobby.host == Some(*you))
        .map(|(lobby, _)| &lobby.rules);
    if accepted_rules.is_some_and(|rules| host_rules.accept_game_rules(rules))
        && let Err(error) = save_host_rule_preferences(host_rules)
    {
        page_error.error = Some(error);
    }
}
