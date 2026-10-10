use super::{UnoJumpInDevice, state::JumpInOpportunity};
use crate::app::runtime::ClientResource;
use crate::app::shell::{UiState, game_command};
use bevy::prelude::*;
use leocard_client::{ItemId, NetworkState, PlayerEconomy};
use leocard_protocol::{UnoCommand, UnoPhaseView};
use leocard_uno::UnoCard;

pub(in super::super) fn update_jump_in_device(
    mut client: Option<ResMut<ClientResource>>,
    economy: Res<PlayerEconomy>,
    mut device: ResMut<UnoJumpInDevice>,
    mut ui: ResMut<UiState>,
) {
    let game = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game());
    let available = game.is_some_and(|game| {
        matches!(game.phase, UnoPhaseView::Playing)
            && game.rules.jump_in_enabled()
            && economy.active(ItemId::UnoJumpInDevice)
    });
    if device.available != available {
        device.available = available;
        ui.dirty = true;
    }
    let connected = client
        .as_deref()
        .is_some_and(|client| matches!(client.0.state(), NetworkState::Connected(_)));
    let opportunity = game.and_then(JumpInOpportunity::from_snapshot);
    if opportunity.is_none() || !connected {
        device.last_attempt = None;
    }
    if game.is_none() {
        device.drawer_open = false;
    }
    if available
        && connected
        && device.enabled
        && let Some(opportunity) = opportunity
        && let Some(client) = client.as_deref_mut()
    {
        send_jump_in(client, &mut device, opportunity.card);
    }
}

/// 手动和自动抢出共用发送入口，避免同一帧发送两次相同请求。
pub(in super::super) fn send_jump_in(
    client: &mut ClientResource,
    device: &mut UnoJumpInDevice,
    card: UnoCard,
) -> bool {
    let Some(opportunity) = client
        .0
        .model()
        .uno_game()
        .and_then(JumpInOpportunity::from_snapshot)
    else {
        return false;
    };
    if opportunity.card != card || device.last_attempt == Some(opportunity) {
        return false;
    }
    if client.0.send(game_command(UnoCommand::JumpIn { card })) {
        device.last_attempt = Some(opportunity);
        true
    } else {
        false
    }
}
