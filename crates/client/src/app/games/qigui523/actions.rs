//! 七鬼五二三规则、提示与出牌按钮动作。

use super::*;
use bevy::ecs::system::SystemParam;
use leocard_protocol::{ClientCommand, GameCommand, PublicPlayRecord, QiGui523Command};
use leocard_qigui523::classify;

#[derive(SystemParam)]
pub struct QiGui523ActionContext<'w, 's> {
    client: Option<ResMut<'w, ClientResource>>,
    ui: ResMut<'w, UiState>,
    no_response_hints: Query<'w, 's, Entity, With<NoLegalResponseHint>>,
    commands: Commands<'w, 's>,
}

pub fn dispatch_qigui523_actions(
    mut actions: MessageReader<PressedUiAction>,
    mut context: QiGui523ActionContext,
) {
    dispatch_domain_actions::<QiGui523UiAction, _>(&mut actions, &mut context);
}

impl UiActionHandler<QiGui523ActionContext<'_, '_>> for QiGui523UiAction {
    fn handle(&self, context: &mut QiGui523ActionContext<'_, '_>) {
        let client = &mut context.client;
        let ui = &mut context.ui;
        match self {
            QiGui523UiAction::UpdateRules(rules) => {
                send_qigui523(client, QiGui523Command::UpdateRules { rules: *rules });
            }
            QiGui523UiAction::Hint => select_hint(client, ui),
            QiGui523UiAction::ToggleCard => {}
            QiGui523UiAction::Play => play_selected(client, ui),
            QiGui523UiAction::Pass => {
                send_qigui523(client, QiGui523Command::Pass);
                for hint in &context.no_response_hints {
                    context.commands.entity(hint).despawn();
                }
            }
        }
    }
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
