//! 七鬼五二三规则、提示与出牌按钮动作。

use super::*;
use leocard_protocol::{ClientCommand, GameCommand, PublicPlayRecord, QiGui523Command};
use leocard_qigui523::classify;

pub fn handle_qigui523_button(
    action: &UiAction,
    client: &mut Option<ResMut<ClientResource>>,
    ui: &mut UiState,
    no_response_hints: &Query<Entity, With<NoLegalResponseHint>>,
    commands: &mut Commands,
) -> bool {
    match action {
        UiAction::UpdateRules(rules) => {
            send_qigui523(client, QiGui523Command::UpdateRules { rules: *rules });
        }
        UiAction::Hint => select_hint(client, ui),
        UiAction::ToggleCard => {}
        UiAction::Play => play_selected(client, ui),
        UiAction::Pass => {
            send_qigui523(client, QiGui523Command::Pass);
            for hint in no_response_hints {
                commands.entity(hint).despawn();
            }
        }
        _ => return false,
    }
    true
}

fn send_qigui523(client: &mut Option<ResMut<ClientResource>>, command: QiGui523Command) {
    if let Some(client) = client.as_deref_mut() {
        client
            .0
            .send(ClientCommand::Game(GameCommand::QiGui523(command)));
    }
}

fn select_hint(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let decision = {
        let model = client.0.model();
        let Some(game) = model.qigui523_game() else {
            return;
        };
        let Some(rules) = model.qigui523_rules() else {
            return;
        };
        let Some(current) = game
            .trick
            .as_ref()
            .and_then(|trick| trick.winning_play.as_ref())
        else {
            return;
        };
        let Ok(current_play) = classify(&current.cards, rules) else {
            return;
        };
        let played_cards = game
            .trick
            .as_ref()
            .into_iter()
            .flat_map(|trick| &trick.records)
            .flat_map(|record| match record {
                PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
                PublicPlayRecord::Passed { .. } => &[],
            })
            .copied()
            .collect::<Vec<_>>();
        next_greedy_hint(
            &mut ui.qigui523.greedy_hint,
            &game.your_hand,
            &current_play,
            &played_cards,
            rules,
        )
    };
    match decision {
        HintDecision::Select(cards) => {
            ui.qigui523.selected.clear();
            ui.qigui523.selected.extend(cards);
        }
        HintDecision::Pass => {
            ui.qigui523.selected.clear();
            client.0.send(ClientCommand::Game(GameCommand::QiGui523(
                QiGui523Command::Pass,
            )));
        }
    }
}

fn play_selected(client: &mut Option<ResMut<ClientResource>>, ui: &mut UiState) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let Some(snapshot) = client.0.model().qigui523_game() else {
        return;
    };
    let cards = ui
        .qigui523
        .selected
        .iter()
        .copied()
        .filter(|card| snapshot.your_hand.contains(card))
        .collect::<Vec<_>>();
    if cards.is_empty() {
        return;
    }
    client.0.send(ClientCommand::Game(GameCommand::QiGui523(
        QiGui523Command::PlayCards { cards },
    )));
    ui.qigui523.selected.clear();
}
