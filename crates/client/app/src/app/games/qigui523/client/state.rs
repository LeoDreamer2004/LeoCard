use crate::app::runtime::GameProjection;
use crate::model::event::EventSequence;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameEvent, GameSnapshot, MatchId, QiGui523Event, QiGui523PhaseView, QiGui523PublicPlay,
    QiGui523PublicPlayRecord, QiGui523Snapshot, QiGui523TrickView, ServerEvent,
};

use leocard_qigui523::QiGuiCard;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScoreCaptureEffect {
    pub player: PlayerId,
    pub cards: Vec<QiGuiCard>,
    /// 每张牌在终局时所属的玩家；`None` 表示牌来自桌面中央。
    pub source_players: Vec<Option<PlayerId>>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct QiGuiClientState {
    pub(super) snapshot: Option<QiGui523Snapshot>,
    pub(crate) captured_score_cards: HashMap<PlayerId, Vec<QiGuiCard>>,
    pub(crate) observed_trick: Option<QiGui523TrickView>,
    play_effect: EventSequence<(PlayerId, QiGui523PublicPlay)>,
    score_capture: EventSequence<ScoreCaptureEffect>,
}

impl QiGuiClientState {
    pub(crate) fn view_key(&self) -> Option<(MatchId, PlayerId)> {
        self.snapshot
            .as_ref()
            .map(|snapshot| (snapshot.match_id, snapshot.you))
    }

    /// 当前这一局中，各玩家已经收入的全部 5、10、K。
    pub fn captured_score_cards(&self, player: PlayerId) -> &[QiGuiCard] {
        self.captured_score_cards
            .get(&player)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn last_play_effect(&self) -> Option<&(PlayerId, QiGui523PublicPlay)> {
        self.play_effect.value.as_ref()
    }

    pub fn play_effect_serial(&self) -> u64 {
        self.play_effect.serial
    }

    pub fn last_score_capture(&self) -> Option<&ScoreCaptureEffect> {
        self.score_capture.value.as_ref()
    }

    pub fn score_capture_serial(&self) -> u64 {
        self.score_capture.serial
    }

    pub(crate) fn apply_qigui523_snapshot(&mut self, snapshot: QiGui523Snapshot) {
        if self.snapshot.as_ref().is_none_or(|previous| {
            previous.match_id != snapshot.match_id || previous.you != snapshot.you
        }) {
            *self = Self::default();
        }
        self.observe_score_cards(&snapshot);
        self.snapshot = Some(snapshot);
    }

    pub(crate) fn apply_qigui523_event(&mut self, event: QiGui523Event) {
        let QiGui523Event::PlayEffect { player, play } = event else {
            return;
        };
        self.play_effect.publish((player, play));
    }

    pub(crate) fn observe_score_cards(&mut self, snapshot: &QiGui523Snapshot) {
        if let QiGui523PhaseView::Finished {
            finisher,
            ref remaining_hands,
            ..
        } = snapshot.phase
        {
            let is_new_finish = !self.snapshot.as_ref().is_some_and(|game| {
                game.match_id == snapshot.match_id
                    && matches!(game.phase, QiGui523PhaseView::Finished { .. })
            });
            if is_new_finish {
                let mut already_captured = self
                    .captured_score_cards
                    .values()
                    .flatten()
                    .copied()
                    .collect::<std::collections::HashSet<_>>();
                let mut visible_cards = self
                    .observed_trick
                    .as_ref()
                    .into_iter()
                    .flat_map(|trick| trick.records.iter())
                    .flat_map(score_cards_in_record)
                    .filter(|card| already_captured.insert(*card))
                    .collect::<Vec<_>>();
                if let Some((player, play)) = &self.play_effect.value
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
                self.captured_score_cards
                    .entry(finisher)
                    .or_default()
                    .extend(visible_cards);
            }
            self.observed_trick = None;
            return;
        }

        if let Some(previous) = self.observed_trick.take() {
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
                self.captured_score_cards
                    .entry(winner)
                    .or_default()
                    .extend(captured);
            }
        }
        self.observed_trick.clone_from(&snapshot.trick);
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
            .find(|state| state.player.id == player)
            .map_or(captured_points, |state| state.score);
        let score_before = self
            .snapshot
            .as_ref()
            .and_then(|game| game.players.iter().find(|state| state.player.id == player))
            .map_or_else(
                || score_after.saturating_sub(captured_points),
                |state| state.score,
            );
        self.score_capture.publish(ScoreCaptureEffect {
            player,
            cards,
            source_players,
            score_before,
            score_after,
        });
    }
}

fn score_cards_in_record(record: &QiGui523PublicPlayRecord) -> Vec<QiGuiCard> {
    match record {
        QiGui523PublicPlayRecord::Played { play, .. } => play
            .cards
            .iter()
            .copied()
            .filter(|card| card.score() > 0)
            .collect(),
        QiGui523PublicPlayRecord::Passed { .. } => Vec::new(),
    }
}

impl GameProjection for QiGuiClientState {
    fn apply(&mut self, update: &ServerEvent) {
        match update {
            ServerEvent::GameSnapshot(GameSnapshot::QiGui523(snapshot)) => {
                self.apply_qigui523_snapshot(snapshot.clone())
            }
            ServerEvent::GameEvent(GameEvent::QiGui523(event)) => {
                self.apply_qigui523_event(event.clone())
            }
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::LeftRoom
            | ServerEvent::RoomClosed => *self = Self::default(),
            _ => {}
        }
    }
}
