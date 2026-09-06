//! 升级按钮动作、本地选牌状态与网络命令。

use super::*;
use leocard_protocol::{
    ClientCommand, GameCommand, ShengjiCommand, ShengjiFiveTrumpCrossingStage, ShengjiPhaseView,
};

pub fn handle_shengji_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    ui: &mut UiState,
    presentation: &mut ShengjiPresentationState,
) -> bool {
    match action {
        UiAction::UpdateShengjiRules(rules) => {
            send_shengji(client, ShengjiCommand::UpdateRules { rules: *rules });
        }
        UiAction::ShengjiDeclare(cards) => send_shengji(
            client,
            ShengjiCommand::Declare {
                cards: cards.clone(),
            },
        ),
        UiAction::ConfirmShengjiBidPass => {
            send_shengji(client, ShengjiCommand::ConfirmBidPass);
        }
        UiAction::ShengjiBottomCopy(cards) => send_shengji(
            client,
            ShengjiCommand::ChooseBottomCopy {
                cards: Some(cards.clone()),
            },
        ),
        UiAction::DeclineBottomCopy => {
            send_shengji(client, ShengjiCommand::ChooseBottomCopy { cards: None });
        }
        UiAction::ToggleShengjiCard => {}
        UiAction::ShengjiHint => select_hint(client, ui),
        UiAction::ShowShengjiPreviousTrick => presentation.reveal_previous_trick(),
        UiAction::ToggleShengjiBuried => {
            ui.shengji.buried_open = !ui.shengji.buried_open;
        }
        UiAction::SubmitShengjiCards => submit_selected_cards(client, ui),
        UiAction::DeclineFiveTrumpCrossing => {
            send_shengji(
                client,
                ShengjiCommand::ChooseFiveTrumpCrossing { cards: None },
            );
            ui.shengji.selected.clear();
        }
        _ => return false,
    }
    true
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
