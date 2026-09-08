//! UNO 按钮动作、本地选择状态与网络命令。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{ClientCommand, GameCommand, UnoCommand, UnoPendingSwapView};
use leocard_uno::UnoFace;

#[derive(SystemParam)]
pub struct UnoActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
}

pub fn dispatch_uno_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: UnoActionContext,
) {
    dispatch_domain_actions::<UnoUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<UnoActionContext<'_>> for UnoUiAction {
    fn handle(&self, context: &mut UnoActionContext<'_>) {
        let client = &mut context.client;
        let ui = &mut context.ui;
        match self {
            UnoUiAction::UpdateRules(rules) => {
                ui.uno.mode_menu_open = false;
                send_uno(client, UnoCommand::UpdateRules { rules: *rules });
            }
            UnoUiAction::ToggleModeMenu => {
                ui.uno.mode_menu_open = !ui.uno.mode_menu_open;
            }
            UnoUiAction::CloseModeMenu => ui.uno.mode_menu_open = false,
            UnoUiAction::ToggleExpansionSettings => {
                ui.uno.expansion_settings_open = !ui.uno.expansion_settings_open;
            }
            UnoUiAction::ToggleCard(card) => {
                let game = client
                    .as_deref()
                    .and_then(|client| client.0.model().uno_game());
                toggle_uno_selection(game, &mut ui.uno.selected, *card);
            }
            UnoUiAction::SubmitCard => submit_selected_cards(client, ui),
            UnoUiAction::CloseColorChoice => ui.uno.color_choice = None,
            UnoUiAction::ChooseInitialColor(color) => {
                send_uno(client, UnoCommand::ChooseInitialColor { color: *color });
            }
            UnoUiAction::PlayCard(card, chosen_color) => {
                send_uno(
                    client,
                    UnoCommand::PlayCard {
                        card: *card,
                        chosen_color: *chosen_color,
                    },
                );
                ui.uno.color_choice = None;
                ui.uno.selected.clear();
            }
            UnoUiAction::JumpIn(card) => {
                send_uno(client, UnoCommand::JumpIn { card: *card });
                ui.uno.selected.clear();
            }
            UnoUiAction::ToggleSwapTarget(target) => {
                let Some(game) = client
                    .as_deref()
                    .and_then(|client| client.0.model().uno_game())
                else {
                    return;
                };
                toggle_uno_swap_target_selection(
                    game.pending_swap,
                    game.you,
                    *target,
                    &mut ui.uno.swap_targets,
                );
                ui.social.interaction_menu_open = None;
            }
            UnoUiAction::ConfirmSwapTargets => confirm_swap_targets(client, ui),
            UnoUiAction::DrawCard => send_uno(client, UnoCommand::DrawCard),
            UnoUiAction::PassAfterDraw => send_uno(client, UnoCommand::PassAfterDraw),
            UnoUiAction::AcceptDrawPenalty => send_uno(client, UnoCommand::AcceptDrawPenalty),
            UnoUiAction::ChallengeDrawFour => send_uno(client, UnoCommand::ChallengeDrawFour),
            UnoUiAction::ResolveSkip => send_uno(client, UnoCommand::ResolveSkip),
            UnoUiAction::Call => send_uno(client, UnoCommand::CallUno),
            UnoUiAction::Report(target) => {
                send_uno(client, UnoCommand::ReportUno { target: *target });
            }
        }
    }
}

fn send_uno(client: &mut Option<ResMut<ClientResource>>, command: UnoCommand) {
    if let Some(client) = client.as_deref_mut() {
        client
            .0
            .send(ClientCommand::Game(GameCommand::Uno(command)));
    }
}

fn submit_selected_cards(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
    let pending_give = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_some_and(|game| {
            matches!(
                game.pending_swap,
                Some(UnoPendingSwapView::SwapOneGive { player, .. }) if player == game.you
            )
        });
    if pending_give && ui.uno.selected.len() == 1 {
        let card = *ui.uno.selected.iter().next().unwrap();
        send_uno(client, UnoCommand::GiveSwapOneCard { card });
        ui.uno.selected.clear();
        return;
    }
    if ui.uno.selected.len() == 1 {
        let card = *ui.uno.selected.iter().next().unwrap();
        if requires_color_choice(card.face()) {
            ui.uno.color_choice = Some(card);
        } else {
            send_uno(
                client,
                UnoCommand::PlayCard {
                    card,
                    chosen_color: None,
                },
            );
            ui.uno.selected.clear();
        }
    } else if ui.uno.selected.len() == 2 {
        let cards = ui.uno.selected.iter().copied().collect();
        send_uno(
            client,
            UnoCommand::PlayCards {
                cards,
                chosen_color: None,
            },
        );
        ui.uno.selected.clear();
    }
}

fn requires_color_choice(face: UnoFace) -> bool {
    matches!(
        face,
        UnoFace::Wild
            | UnoFace::DarkWild
            | UnoFace::WildDrawTwo
            | UnoFace::WildDrawFour
            | UnoFace::WildDrawColor
            | UnoFace::WildPowerReverse
            | UnoFace::WildNoU
            | UnoFace::WildStackThree
            | UnoFace::WildStackNumber
            | UnoFace::WildReverseDrawFour
            | UnoFace::WildDrawSix
            | UnoFace::WildDrawTen
    )
}

fn confirm_swap_targets(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    else {
        return;
    };
    let command = match game.pending_swap {
        Some(UnoPendingSwapView::SwapOneTarget { player })
            if player == game.you && ui.uno.swap_targets.len() == 1 =>
        {
            Some(UnoCommand::ChooseSwapOneTarget {
                target: ui.uno.swap_targets[0],
            })
        }
        Some(UnoPendingSwapView::ForceTrade { player })
            if player == game.you && ui.uno.swap_targets.len() == 2 =>
        {
            Some(UnoCommand::ForceTradeHands {
                first: ui.uno.swap_targets[0],
                second: ui.uno.swap_targets[1],
            })
        }
        Some(UnoPendingSwapView::SevenSwap { player })
            if player == game.you && ui.uno.swap_targets.len() == 1 =>
        {
            Some(UnoCommand::ChooseSevenSwapTarget {
                target: ui.uno.swap_targets[0],
            })
        }
        _ => None,
    };
    if let Some(command) = command {
        send_uno(client, command);
        ui.uno.swap_targets.clear();
    }
}
