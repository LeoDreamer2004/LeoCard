//! 玩家互动菜单与跨游戏托管按钮动作。

use super::super::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
    game_command,
};
use super::{AutoEggThrower, PlayerInteractionCooldown};
use crate::app::runtime::{ClientResource, PageErrorState};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::PlayerEconomy;
use leocard_game_common::PlayerId;
use leocard_protocol::{ClientCommand, PlayerInteractionKind};

#[derive(Clone)]
pub(crate) enum SocialUiAction {
    SpectatePlayer(PlayerId),
    ToggleInteractionMenu {
        player: PlayerId,
        anchor: Entity,
    },
    ToggleAutoPlay,
    UseAutoEggThrower(PlayerId),
    SendInteraction {
        target: PlayerId,
        kind: PlayerInteractionKind,
    },
}

impl DomainUiAction for SocialUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Social(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        false
    }
}

#[derive(SystemParam)]
pub(crate) struct SocialActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
    cooldown: ResMut<'w, PlayerInteractionCooldown>,
    thrower: ResMut<'w, AutoEggThrower>,
    economy: ResMut<'w, PlayerEconomy>,
    error: ResMut<'w, PageErrorState>,
    time: Res<'w, Time<Real>>,
}

pub(crate) fn dispatch_social_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: SocialActionContext,
) {
    dispatch_domain_actions::<SocialUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<SocialActionContext<'_>> for SocialUiAction {
    fn handle(&self, context: &mut SocialActionContext<'_>) {
        let client = &mut context.client;
        let ui = &mut *context.ui;
        let cooldown = &mut *context.cooldown;
        match self {
            Self::SpectatePlayer(player) => {
                if let Some(client) = client.as_deref_mut() {
                    client
                        .0
                        .send(ClientCommand::SpectatePlayer { player: *player });
                }
            }
            SocialUiAction::ToggleInteractionMenu { player, anchor } => {
                ui.social.interaction_menu_open = (ui.social.interaction_menu_open
                    != Some(*player)
                    || ui.social.interaction_menu_anchor != Some(*anchor))
                .then_some(*player);
                ui.social.interaction_menu_anchor = Some(*anchor);
            }
            Self::UseAutoEggThrower(target) => {
                let result = if let Some(client) = client.as_deref() {
                    context.thrower.activate(
                        &client.0,
                        *target,
                        context.time.elapsed(),
                        &mut context.economy,
                    )
                } else {
                    Err("尚未连接房间，无法使用自动砸蛋器".to_owned())
                };
                match result {
                    Ok(()) => ui.social.interaction_menu_open = None,
                    Err(error) => context.error.error = Some(error),
                }
            }
            SocialUiAction::SendInteraction { target, kind } => {
                if cooldown.is_active(*kind) {
                    return;
                }
                if let Some(client) = client.as_deref_mut()
                    && client.0.send(ClientCommand::Interact {
                        target: *target,
                        kind: *kind,
                    })
                {
                    let duration = match kind {
                        PlayerInteractionKind::Flower | PlayerInteractionKind::Egg => 0.5,
                        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe => 5.0,
                    };
                    cooldown.start(*kind, duration);
                    if matches!(
                        kind,
                        PlayerInteractionKind::Wine | PlayerInteractionKind::Shoe
                    ) {
                        ui.social.interaction_menu_open = None;
                    }
                }
            }
            SocialUiAction::ToggleAutoPlay => toggle_auto_play(client),
        }
    }
}

fn toggle_auto_play(client: &mut Option<ResMut<ClientResource>>) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(command) = client.0.model().toggle_auto_play_command() else {
        return;
    };
    client.0.send(game_command(command));
}
