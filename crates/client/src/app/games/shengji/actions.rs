//! 升级按钮动作、本地选牌状态与网络命令。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{
    ClientCommand, GameCommand, ShengjiCommand, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView,
};

#[derive(SystemParam)]
pub struct ShengjiActionContext<'w> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
    presentation: ResMut<'w, ShengjiPresentationState>,
}

pub fn dispatch_shengji_actions(
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
                send_shengji(client, ShengjiCommand::UpdateRules { rules: *rules });
            }
            ShengjiUiAction::Declare(cards) => send_shengji(
                client,
                ShengjiCommand::Declare {
                    cards: cards.clone(),
                },
            ),
            ShengjiUiAction::ConfirmBidPass => {
                send_shengji(client, ShengjiCommand::ConfirmBidPass);
            }
            ShengjiUiAction::BottomCopy(cards) => send_shengji(
                client,
                ShengjiCommand::ChooseBottomCopy {
                    cards: Some(cards.clone()),
                },
            ),
            ShengjiUiAction::DeclineBottomCopy => {
                send_shengji(client, ShengjiCommand::ChooseBottomCopy { cards: None });
            }
            ShengjiUiAction::ToggleCard => {}
            ShengjiUiAction::Hint => select_hint(client, ui),
            ShengjiUiAction::ShowPreviousTrick => context.presentation.reveal_previous_trick(),
            ShengjiUiAction::ToggleBuried => {
                ui.shengji.buried_open = !ui.shengji.buried_open;
            }
            ShengjiUiAction::SubmitCards => submit_selected_cards(client, ui),
            ShengjiUiAction::DeclineFiveTrumpCrossing => {
                send_shengji(
                    client,
                    ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
                );
                ui.shengji.selected.clear();
            }
        }
    }
}

fn send_shengji(client: &mut Option<ResMut<ClientResource>>, command: ShengjiCommand) {
    if let Some(client) = client.as_deref_mut() {
        client
            .0
            .send(ClientCommand::Game(GameCommand::Shengji(command)));
    }
}

fn select_hint(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    else {
        return;
    };
    let Some(cards) = next_shengji_hint(game, &ui.shengji.selected) else {
        return;
    };
    ui.shengji.selected.clear();
    ui.shengji.selected.extend(cards);
}

fn submit_selected_cards(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
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
        .filter(|card| ui.shengji.selected.contains(card))
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
    send_shengji(client, command);
    ui.shengji.selected.clear();
}
