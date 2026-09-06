use std::collections::HashMap;

use leocard_protocol::{
    GamePhaseView, GameSnapshot, PlayerId, PublicPlay, PublicPlayRecord, QiGui523Event,
    QiGui523Snapshot, TrickView,
};
use leocard_qigui523::QiGuiCard;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScoreCaptureEffect {
    pub player: PlayerId,
    pub cards: Vec<QiGuiCard>,
    /// 每张牌在终局时所属的玩家；`None` 表示牌来自桌面中央。
    pub source_players: Vec<Option<PlayerId>>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Clone, Debug, Default)]
pub(super) struct QiGuiClientState {
    pub(super) captured_score_cards: HashMap<PlayerId, Vec<QiGuiCard>>,
    pub(super) observed_trick: Option<TrickView>,
    play_effect: Sequenced<(PlayerId, PublicPlay)>,
    score_capture: Sequenced<ScoreCaptureEffect>,
}

impl ClientModel {
    /// 当前这一局中，各玩家已经收入的全部 5、10、K。
    pub fn captured_score_cards(&self, player: PlayerId) -> &[QiGuiCard] {
        self.games
            .qigui523
            .captured_score_cards
            .get(&player)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn last_play_effect(&self) -> Option<&(PlayerId, PublicPlay)> {
        self.games.qigui523.play_effect.value.as_ref()
    }

    pub fn play_effect_serial(&self) -> u64 {
        self.games.qigui523.play_effect.serial
    }

    pub fn last_score_capture(&self) -> Option<&ScoreCaptureEffect> {
        self.games.qigui523.score_capture.value.as_ref()
    }

    pub fn score_capture_serial(&self) -> u64 {
        self.games.qigui523.score_capture.serial
    }

    pub(super) fn reset_qigui523_tracking(&mut self) {
        self.games.qigui523.captured_score_cards.clear();
        self.games.qigui523.observed_trick = None;
        self.games.qigui523.score_capture.clear();
    }

    pub(super) fn apply_qigui523_snapshot(&mut self, snapshot: QiGui523Snapshot) {
        self.host_port = Some(snapshot.host_port);
        if self.active_match_id != Some(snapshot.match_id) {
            self.reset_qigui523_tracking();
            self.active_match_id = Some(snapshot.match_id);
            self.pending.player_interactions.clear();
        }
        if let GamePhaseView::Finished {
            match_id,
            reference_changes,
            ..
        } = &snapshot.phase
        {
            self.last_finished_match = Some((*match_id, reference_changes.clone()));
        }
        self.observe_score_cards(&snapshot);
        self.you = Some(snapshot.you);
        self.game = Some(GameSnapshot::QiGui523(snapshot));
        self.lobby = None;
        self.rejection.value = None;
    }

    pub(super) fn apply_qigui523_event(&mut self, event: QiGui523Event) {
        let QiGui523Event::PlayEffect { player, play } = event;
        self.games.qigui523.play_effect.publish((player, play));
    }

    pub(super) fn observe_score_cards(&mut self, snapshot: &QiGui523Snapshot) {
        if let GamePhaseView::Finished {
            finisher,
            ref remaining_hands,
            ..
        } = snapshot.phase
        {
            let is_new_finish = !self.qigui523_game().is_some_and(|game| {
                game.match_id == snapshot.match_id
                    && matches!(game.phase, GamePhaseView::Finished { .. })
            });
            if is_new_finish {
                let mut already_captured = self
                    .games
                    .qigui523
                    .captured_score_cards
                    .values()
                    .flatten()
                    .copied()
                    .collect::<std::collections::HashSet<_>>();
                let mut visible_cards = self
                    .games
                    .qigui523
                    .observed_trick
                    .as_ref()
                    .into_iter()
                    .flat_map(|trick| trick.records.iter())
                    .flat_map(score_cards_in_record)
                    .filter(|card| already_captured.insert(*card))
                    .collect::<Vec<_>>();
                if let Some((player, play)) = &self.games.qigui523.play_effect.value
                    && *player == finisher
                {
                    for card in play.cards.iter().copied().filter(|card| card.score() > 0) {
                        if already_captured.insert(card) {
                            visible_cards.push(card);
                        }
                    }
                }
                let mut source_players = vec![None; visible_cards.len()];
                for hand in remaining_hands {
                    for card in hand.cards.iter().copied().filter(|card| card.score() > 0) {
                        if !already_captured.insert(card) {
                            continue;
                        }
                        visible_cards.push(card);
                        source_players.push(Some(hand.player));
                    }
                }
                self.record_score_capture(
                    snapshot,
                    finisher,
                    visible_cards.clone(),
                    source_players,
                );
                self.games
                    .qigui523
                    .captured_score_cards
                    .entry(finisher)
                    .or_default()
                    .extend(visible_cards);
            }
            self.games.qigui523.observed_trick = None;
            return;
        }

        if let Some(previous) = self.games.qigui523.observed_trick.take() {
            let is_same_trick = snapshot.trick.as_ref().is_some_and(|current| {
                current.leader == previous.leader && current.records.starts_with(&previous.records)
            });
            if !is_same_trick && let Some(winner) = previous.winning_player {
                let captured = previous
                    .records
                    .iter()
                    .flat_map(score_cards_in_record)
                    .collect::<Vec<_>>();
                self.record_score_capture(
                    snapshot,
                    winner,
                    captured.clone(),
                    vec![None; captured.len()],
                );
                self.games
                    .qigui523
                    .captured_score_cards
                    .entry(winner)
                    .or_default()
                    .extend(captured);
            }
        }
        self.games
            .qigui523
            .observed_trick
            .clone_from(&snapshot.trick);
    }

    fn record_score_capture(
        &mut self,
        snapshot: &QiGui523Snapshot,
        player: PlayerId,
        cards: Vec<QiGuiCard>,
        source_players: Vec<Option<PlayerId>>,
    ) {
        if cards.is_empty() {
            return;
        }
        let captured_points = cards
            .iter()
            .copied()
            .map(QiGuiCard::score)
            .map(u32::from)
            .sum::<u32>();
        let score_after = snapshot
            .players
            .iter()
            .find(|state| state.id == player)
            .map_or(captured_points, |state| state.score);
        let score_before = self
            .qigui523_game()
            .and_then(|game| game.players.iter().find(|state| state.id == player))
            .map_or_else(
                || score_after.saturating_sub(captured_points),
                |state| state.score,
            );
        self.games
            .qigui523
            .score_capture
            .publish(ScoreCaptureEffect {
                player,
                cards,
                source_players,
                score_before,
                score_after,
            });
    }
}

fn score_cards_in_record(record: &PublicPlayRecord) -> Vec<QiGuiCard> {
    match record {
        PublicPlayRecord::Played { play, .. } => play
            .cards
            .iter()
            .copied()
            .filter(|card| card.score() > 0)
            .collect(),
        PublicPlayRecord::Passed { .. } => Vec::new(),
    }
}
