//! UNO 按钮动作、本地选择状态与网络命令。

use super::{UnoUiState, toggle_uno_selection, toggle_uno_swap_target_selection};
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, UiState, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, UnoCommand, UnoPendingSwapView};
use leocard_uno::{UnoCard, UnoColor, UnoFace, UnoRuleSet};

#[derive(Clone)]
pub(crate) enum UnoUiAction {
    UpdateRules(UnoRuleSet),
    ToggleModeMenu,
    CloseModeMenu,
    ToggleExpansionSettings,
    ToggleCard(UnoCard),
    SubmitCard,
    CloseColorChoice,
    ChooseInitialColor(UnoColor),
    PlayCard(UnoCard, Option<UnoColor>),
    JumpIn(UnoCard),
    ToggleSwapTarget(PlayerId),
    ConfirmSwapTargets,
    DrawCard,
    PassAfterDraw,
    AcceptDrawPenalty,
    ChallengeDrawFour,
    ResolveSkip,
    Call,
    Report(PlayerId),
}

impl DomainUiAction for UnoUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Uno(action) = action else {
            return None;
        };
        Some(action)
    }
}

#[derive(SystemParam)]
pub(crate) struct UnoActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    game_ui: ResMut<'w, UnoUiState>,
    ui: ResMut<'w, UiState>,
}

pub(super) fn dispatch_uno_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: UnoActionContext,
) {
    dispatch_domain_actions::<UnoUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<UnoActionContext<'_>> for UnoUiAction {
    fn handle(&self, context: &mut UnoActionContext<'_>) {
        let client = &mut context.client;
        let ui = &mut context.game_ui;
        match self {
            UnoUiAction::UpdateRules(rules) => {
                ui.mode_menu_open = false;
                send_game_command(client, UnoCommand::UpdateRules { rules: *rules });
            }
            UnoUiAction::ToggleModeMenu => {
                ui.mode_menu_open = !ui.mode_menu_open;
            }
            UnoUiAction::CloseModeMenu => ui.mode_menu_open = false,
            UnoUiAction::ToggleExpansionSettings => {
                ui.expansion_settings_open = !ui.expansion_settings_open;
            }
            UnoUiAction::ToggleCard(card) => {
                let game = client
                    .as_deref()
                    .and_then(|client| client.0.model().uno_game());
                toggle_uno_selection(game, &mut ui.selected, *card);
            }
            UnoUiAction::SubmitCard => submit_selected_cards(client, ui),
            UnoUiAction::CloseColorChoice => ui.color_choice = None,
            UnoUiAction::ChooseInitialColor(color) => {
                send_game_command(client, UnoCommand::ChooseInitialColor { color: *color });
            }
            UnoUiAction::PlayCard(card, chosen_color) => {
                send_game_command(
                    client,
                    UnoCommand::PlayCard {
                        card: *card,
                        chosen_color: *chosen_color,
                    },
                );
                ui.color_choice = None;
                ui.selected.clear();
            }
            UnoUiAction::JumpIn(card) => {
                send_game_command(client, UnoCommand::JumpIn { card: *card });
                ui.selected.clear();
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
                    &mut ui.swap_targets,
                );
                context.ui.social.interaction_menu_open = None;
            }
            UnoUiAction::ConfirmSwapTargets => confirm_swap_targets(client, ui),
            UnoUiAction::DrawCard => send_game_command(client, UnoCommand::DrawCard),
            UnoUiAction::PassAfterDraw => send_game_command(client, UnoCommand::PassAfterDraw),
            UnoUiAction::AcceptDrawPenalty => {
                send_game_command(client, UnoCommand::AcceptDrawPenalty);
            }
            UnoUiAction::ChallengeDrawFour => {
                send_game_command(client, UnoCommand::ChallengeDrawFour);
            }
            UnoUiAction::ResolveSkip => send_game_command(client, UnoCommand::ResolveSkip),
            UnoUiAction::Call => send_game_command(client, UnoCommand::CallUno),
            UnoUiAction::Report(target) => {
                send_game_command(client, UnoCommand::ReportUno { target: *target });
            }
        }
    }
}

fn submit_selected_cards(client: &mut Option<ResMut<ClientResource>>, ui: &mut UnoUiState) {
    let pending_give = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_some_and(|game| {
            matches!(
                game.pending_swap,
                Some(UnoPendingSwapView::SwapOneGive { player, .. }) if player == game.you
            )
        });
    if pending_give && ui.selected.len() == 1 {
        let card = *ui.selected.iter().next().unwrap();
        send_game_command(client, UnoCommand::GiveSwapOneCard { card });
        ui.selected.clear();
        return;
    }
    if ui.selected.len() == 1 {
        let card = *ui.selected.iter().next().unwrap();
        if requires_color_choice(card.face()) {
            ui.color_choice = Some(card);
        } else {
            send_game_command(
                client,
                UnoCommand::PlayCard {
                    card,
                    chosen_color: None,
                },
            );
            ui.selected.clear();
        }
    } else if ui.selected.len() == 2 {
        let cards = ui.selected.iter().copied().collect();
        send_game_command(
            client,
            UnoCommand::PlayCards {
                cards,
                chosen_color: None,
            },
        );
        ui.selected.clear();
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

fn confirm_swap_targets(client: &mut Option<ResMut<ClientResource>>, ui: &mut UnoUiState) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    else {
        return;
    };
    let command = match game.pending_swap {
        Some(UnoPendingSwapView::SwapOneTarget { player })
            if player == game.you && ui.swap_targets.len() == 1 =>
        {
            Some(UnoCommand::ChooseSwapOneTarget {
                target: ui.swap_targets[0],
            })
        }
        Some(UnoPendingSwapView::ForceTrade { player })
            if player == game.you && ui.swap_targets.len() == 2 =>
        {
            Some(UnoCommand::ForceTradeHands {
                first: ui.swap_targets[0],
                second: ui.swap_targets[1],
            })
        }
        Some(UnoPendingSwapView::SevenSwap { player })
            if player == game.you && ui.swap_targets.len() == 1 =>
        {
            Some(UnoCommand::ChooseSevenSwapTarget {
                target: ui.swap_targets[0],
            })
        }
        _ => None,
    };
    if let Some(command) = command {
        send_game_command(client, command);
        ui.swap_targets.clear();
    }
}
