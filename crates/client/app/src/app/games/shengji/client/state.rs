use crate::app::runtime::GameProjection;
use crate::model::event::{EventInbox, EventSequence};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{
    GameEvent, GameSnapshot, MatchId, ServerEvent, ShengjiEvent, ShengjiPhaseView, ShengjiSnapshot,
};

use leocard_shengji::ShengjiCard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShengjiScoreCaptureEffect {
    pub cards: Vec<ShengjiCard>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct ShengjiClientState {
    pub(super) snapshot: Option<ShengjiSnapshot>,
    collected_score_cards: Vec<ShengjiCard>,
    pending_trick_score_cards: Vec<ShengjiCard>,
    finished_event_pending_snapshot: bool,
    score_capture: EventSequence<ShengjiScoreCaptureEffect>,
    events: EventInbox<ShengjiEvent>,
}

impl ShengjiClientState {
    pub(crate) fn view_key(&self) -> Option<(MatchId, PlayerId)> {
        self.snapshot
            .as_ref()
            .map(|snapshot| (snapshot.match_id, snapshot.you))
    }

    pub fn shengji_collected_score_cards(&self) -> &[ShengjiCard] {
        &self.collected_score_cards
    }

    pub fn last_shengji_score_capture(&self) -> Option<&ShengjiScoreCaptureEffect> {
        self.score_capture.value.as_ref()
    }

    pub fn shengji_score_capture_serial(&self) -> u64 {
        self.score_capture.serial
    }

    pub fn take_shengji_events(&mut self) -> Vec<ShengjiEvent> {
        self.events.take()
    }

    fn reset_shengji_tracking(&mut self) {
        self.collected_score_cards.clear();
        self.pending_trick_score_cards.clear();
        self.finished_event_pending_snapshot = false;
        self.score_capture.clear();
    }

    pub(crate) fn apply_shengji_snapshot(&mut self, snapshot: ShengjiSnapshot) {
        if self.snapshot.as_ref().is_none_or(|previous| {
            previous.match_id != snapshot.match_id || previous.you != snapshot.you
        }) {
            self.events.clear();
            *self = Self::default();
        } else {
            if self
                .snapshot
                .as_ref()
                .is_some_and(|previous| previous.hand_number != snapshot.hand_number)
            {
                self.reset_shengji_tracking();
            }
            let captured = if self.finished_event_pending_snapshot {
                self.finished_event_pending_snapshot = false;
                Vec::new()
            } else {
                self.snapshot.as_ref().map_or_else(Vec::new, |previous| {
                    completed_shengji_trick_score_cards(previous, &snapshot)
                })
            };
            for card in captured {
                if !self.collected_score_cards.contains(&card) {
                    self.collected_score_cards.push(card);
                }
            }
        }
        self.snapshot = Some(snapshot);
    }

    pub(crate) fn apply_shengji_event(&mut self, event: ShengjiEvent) {
        match &event {
            ShengjiEvent::CardsPlayed { play, .. } => {
                for card in play
                    .play
                    .cards
                    .iter()
                    .copied()
                    .filter(|card| card.points() > 0)
                {
                    if !self.pending_trick_score_cards.contains(&card) {
                        self.pending_trick_score_cards.push(card);
                    }
                }
            }
            ShengjiEvent::TrickFinished {
                winner,
                collecting_score,
                ..
            } => {
                let collecting_side_won = self
                    .snapshot
                    .as_ref()
                    .and_then(|game| game.dealer)
                    .is_some_and(|dealer| winner.0 % 2 != dealer.0 % 2);
                if collecting_side_won {
                    let cards = std::mem::take(&mut self.pending_trick_score_cards);
                    if !cards.is_empty() {
                        let score_before = self
                            .snapshot
                            .as_ref()
                            .map_or(0, |game| game.collecting_score);
                        self.score_capture.publish(ShengjiScoreCaptureEffect {
                            cards: cards.clone(),
                            score_before,
                            score_after: *collecting_score,
                        });
                    }
                    for card in cards {
                        if !self.collected_score_cards.contains(&card) {
                            self.collected_score_cards.push(card);
                        }
                    }
                } else {
                    self.pending_trick_score_cards.clear();
                }
                self.finished_event_pending_snapshot = true;
            }
            ShengjiEvent::RedealRequired { .. } => {
                self.pending_trick_score_cards.clear();
                self.finished_event_pending_snapshot = false;
            }
            _ => {}
        }
        self.events.push(event);
    }
}

fn completed_shengji_trick_score_cards(
    previous: &ShengjiSnapshot,
    next: &ShengjiSnapshot,
) -> Vec<ShengjiCard> {
    let Some(trick) = previous.trick.as_ref() else {
        return Vec::new();
    };
    let same_trick_continues = next.trick.as_ref().is_some_and(|current| {
        current.leader == trick.leader
            && current.plays.len() >= trick.plays.len()
            && current.plays[..trick.plays.len()] == trick.plays
    });
    let just_finished = matches!(next.phase, ShengjiPhaseView::Finished { .. })
        && !matches!(previous.phase, ShengjiPhaseView::Finished { .. });
    if same_trick_continues && !just_finished {
        return Vec::new();
    }
    let Some(dealer) = previous.dealer.or(next.dealer) else {
        return Vec::new();
    };
    if trick.winning_player.0 % 2 == dealer.0 % 2 {
        return Vec::new();
    }
    trick
        .plays
        .iter()
        .flat_map(|played| played.play.cards.iter().copied())
        .filter(|card| card.points() > 0)
        .collect()
}

impl GameProjection for ShengjiClientState {
    fn apply(&mut self, update: &ServerEvent) {
        match update {
            ServerEvent::GameSnapshot(GameSnapshot::Shengji(snapshot)) => {
                self.apply_shengji_snapshot(snapshot.clone())
            }
            ServerEvent::GameEvent(GameEvent::Shengji(event)) => {
                self.apply_shengji_event(event.clone())
            }
            ServerEvent::GameSnapshot(_)
            | ServerEvent::LobbySnapshot(_)
            | ServerEvent::LeftRoom
            | ServerEvent::RoomClosed
            | ServerEvent::KickedFromRoom => *self = Self::default(),
            _ => {}
        }
    }
}
