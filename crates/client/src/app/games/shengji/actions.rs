//! 升级按钮动作、本地选牌状态与网络命令。

use super::{ShengjiPresentationState, ShengjiUiState, next_shengji_hint};
use crate::app::runtime::ClientResource;
use crate::app::shell::{
    DomainUiAction, PressedUiAction, UiAction, UiActionHandler, dispatch_domain_actions,
    send_game_command,
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_protocol::{ShengjiCommand, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView};
use leocard_shengji::{ShengjiCard, ShengjiRuleSet};

#[derive(Clone)]
pub(crate) enum ShengjiUiAction {
    UpdateRules(ShengjiRuleSet),
    Declare(Vec<ShengjiCard>),
    ConfirmBidPass,
    BottomCopy(Vec<ShengjiCard>),
    DeclineBottomCopy,
    ToggleCard,
    Hint,
    ShowPreviousTrick,
    ToggleBuried,
    SubmitCards,
    DeclineFiveTrumpCrossing,
}

impl DomainUiAction for ShengjiUiAction {
    fn extract(action: &UiAction) -> Option<&Self> {
        let UiAction::Shengji(action) = action else {
            return None;
        };
        Some(action)
    }

    fn rebuilds_ui(&self) -> bool {
        !matches!(self, Self::ToggleCard)
    }
}

#[derive(SystemParam)]
pub(crate) struct ShengjiActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, ShengjiUiState>,
    presentation: ResMut<'w, ShengjiPresentationState>,
}

pub(super) fn dispatch_shengji_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: ShengjiActionContext,
) {
    dispatch_domain_actions::<ShengjiUiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<ShengjiActionContext<'_>> for ShengjiUiAction {
    fn handle(&self, context: &mut ShengjiActionContext<'_>) {
        let client = &mut context.client;
        let ui = &mut context.ui;
        match self {
            ShengjiUiAction::UpdateRules(rules) => {
                send_game_command(client, ShengjiCommand::UpdateRules { rules: *rules });
            }
            ShengjiUiAction::Declare(cards) => send_game_command(
                client,
                ShengjiCommand::Declare {
                    cards: cards.clone(),
                },
            ),
            ShengjiUiAction::ConfirmBidPass => {
                send_game_command(client, ShengjiCommand::ConfirmBidPass);
            }
            ShengjiUiAction::BottomCopy(cards) => send_game_command(
                client,
                ShengjiCommand::ChooseBottomCopy {
                    cards: Some(cards.clone()),
                },
            ),
            ShengjiUiAction::DeclineBottomCopy => {
                send_game_command(client, ShengjiCommand::ChooseBottomCopy { cards: None });
            }
            ShengjiUiAction::ToggleCard => {}
            ShengjiUiAction::Hint => select_hint(client, ui),
            ShengjiUiAction::ShowPreviousTrick => context.presentation.reveal_previous_trick(),
            ShengjiUiAction::ToggleBuried => {
                ui.buried_open = !ui.buried_open;
            }
            ShengjiUiAction::SubmitCards => submit_selected_cards(client, ui),
            ShengjiUiAction::DeclineFiveTrumpCrossing => {
                send_game_command(
                    client,
                    ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                );
                ui.selected.clear();
            }
        }
    }
}

fn select_hint(client: &mut Option<ResMut<ClientResource>>, ui: &mut ShengjiUiState) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        return;
    };
    let Some(cards) = next_shengji_hint(game, &ui.selected) else {
        return;
    };
    ui.selected.clear();
    ui.selected.extend(cards);
}

fn submit_selected_cards(client: &mut Option<ResMut<ClientResource>>, ui: &mut ShengjiUiState) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        return;
    };
    let cards = game
        .your_hand
        .iter()
        .copied()
        .filter(|card| ui.selected.contains(card))
        .collect::<Vec<_>>();
    let command = match game.phase {
        ShengjiPhaseView::Burying | ShengjiPhaseView::BottomCopyBurying { .. } => {
            ShengjiCommand::Bury { cards }
        }
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Deciding,
            ..
        } => ShengjiCommand::ChooseFiveTrumpCrossing { cards: Some(cards) },
        ShengjiPhaseView::FiveTrumpCrossing {
            stage: ShengjiFiveTrumpCrossingStage::Returning,
            ..
        } => ShengjiCommand::ReturnFiveTrumpCrossing { cards },
        ShengjiPhaseView::Playing => ShengjiCommand::PlayCards { cards },
        _ => return,
    };
    send_game_command(client, command);
    ui.selected.clear();
}
