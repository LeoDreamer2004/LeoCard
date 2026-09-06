//! 七鬼五二三的提示策略与无牌可压反馈。

use super::*;
use leocard_protocol::PublicPlayRecord;
use leocard_protocol::QiGui523Snapshot;
use leocard_qigui523::{
    ClassifiedPlay, QiGui523Bot, QiGui523BotRequest, QiGuiCard, QiGuiRuleSet, classify,
    has_legal_response,
};

#[derive(Debug, Eq, PartialEq)]
pub enum HintDecision {
    Select(Vec<QiGuiCard>),
    Pass,
}

#[derive(Component)]
pub struct PlaySelectionCount;

#[derive(Component)]
pub struct NoLegalResponseHint;

pub fn next_greedy_hint(
    strategy: &mut QiGui523Bot,
    hand: &[QiGuiCard],
    current_play: &ClassifiedPlay,
    played_cards: &[QiGuiCard],
    rules: &QiGuiRuleSet,
) -> HintDecision {
    let request = QiGui523BotRequest {
        hand,
        current_play,
        played_cards,
        rules,
    };
    if let Some(play) = strategy.choose(request) {
        return HintDecision::Select(play.cards().to_vec());
    }

    strategy.reset();
    strategy.choose(request).map_or(HintDecision::Pass, |play| {
        HintDecision::Select(play.cards().to_vec())
    })
}

pub fn game_has_legal_response(game: &QiGui523Snapshot, rules: &QiGuiRuleSet) -> bool {
    let Some(trick) = game.trick.as_ref() else {
        return false;
    };
    let Some(current) = trick.winning_play.as_ref() else {
        return true;
    };
    let Ok(current_play) = classify(&current.cards, rules) else {
        return false;
    };
    let played_cards = trick
        .records
        .iter()
        .flat_map(|record| match record {
            PublicPlayRecord::Played { play, .. } => play.cards.as_slice(),
            PublicPlayRecord::Passed { .. } => &[],
        })
        .copied()
        .collect::<Vec<_>>();
    has_legal_response(QiGui523BotRequest {
        hand: &game.your_hand,
        current_play: &current_play,
        played_cards: &played_cards,
        rules,
    })
}

pub fn sync_selection_label(
    ui: Res<UiState>,
    mut labels: Query<&mut Text, With<PlaySelectionCount>>,
) {
    let expected = format!("出牌 ({})", ui.qigui523.selected.len());
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}

pub fn animate_no_legal_response_hint(
    time: Res<Time>,
    mut hints: Query<&mut UiTransform, With<NoLegalResponseHint>>,
) {
    let offset = (time.elapsed_secs() * 3.2).sin() * 3.0;
    for mut transform in &mut hints {
        transform.translation = Val2::px(0.0, offset);
    }
}
