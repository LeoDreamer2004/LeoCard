//! 玩家互动菜单与跨游戏托管按钮动作。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{
    ClientCommand, GameCommand, PlayerInteractionKind, QiGui523Command, ShengjiCommand,
    TexasHoldemCommand, UnoCommand,
};

#[derive(SystemParam)]
pub struct SocialActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
    cooldown: ResMut<'w, PlayerInteractionCooldown>,
}

pub fn dispatch_social_actions(
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
            SocialUiAction::ToggleInteractionMenu(player) => {
                ui.social.interaction_menu_open =
                    (ui.social.interaction_menu_open != Some(*player)).then_some(*player);
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
    let command = if let Some(game) = client.0.model().qigui523_game() {
        let enabled = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.auto_play);
        GameCommand::QiGui523(QiGui523Command::SetAutoPlay { enabled: !enabled })
    } else if let Some(game) = client.0.model().texas_holdem_game() {
        let enabled = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.auto_play);
        GameCommand::TexasHoldem(TexasHoldemCommand::SetAutoPlay { enabled: !enabled })
    } else if let Some(game) = client.0.model().shengji_game() {
        let enabled = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.auto_play);
        GameCommand::Shengji(ShengjiCommand::SetAutoPlay { enabled: !enabled })
    } else if let Some(game) = client.0.model().uno_game() {
        let enabled = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.auto_play);
        GameCommand::Uno(UnoCommand::SetAutoPlay { enabled: !enabled })
    } else {
        return;
    };
    client.0.send(ClientCommand::Game(command));
}
