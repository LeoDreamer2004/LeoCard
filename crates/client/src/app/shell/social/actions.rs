//! 玩家互动菜单与跨游戏托管按钮动作。

use super::*;
use leocard_protocol::{
    ClientCommand, GameCommand, PlayerInteractionKind, QiGui523Command, ShengjiCommand,
    TexasHoldemCommand, UnoCommand,
};

pub fn handle_social_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    ui: &mut UiState,
    cooldown: &mut PlayerInteractionCooldown,
) -> bool {
    match action {
        UiAction::ToggleInteractionMenu(player) => {
            ui.social.interaction_menu_open =
                (ui.social.interaction_menu_open != Some(*player)).then_some(*player);
        }
        UiAction::SendInteraction { target, kind } => {
            if cooldown.is_active(*kind) {
                return true;
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
        UiAction::ToggleAutoPlay => toggle_auto_play(client),
        _ => return false,
    }
    true
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
