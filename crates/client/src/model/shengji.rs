use super::ClientModel;
use super::types::{GameEventInbox, Sequenced};
use leocard_protocol::{GameRules, ShengjiEvent, ShengjiPhaseView, ShengjiSnapshot};
use leocard_shengji::ShengjiCard;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShengjiScoreCaptureEffect {
    pub cards: Vec<ShengjiCard>,
    pub score_before: u32,
    pub score_after: u32,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ShengjiClientState {
    collected_score_cards: Vec<ShengjiCard>,
    pending_trick_score_cards: Vec<ShengjiCard>,
    finished_event_pending_snapshot: bool,
    score_capture: Sequenced<ShengjiScoreCaptureEffect>,
    events: GameEventInbox<ShengjiEvent>,
}

impl ClientModel {
    pub fn shengji_collected_score_cards(&self) -> &[ShengjiCard] {
        &self.games.shengji.collected_score_cards
    }

    pub fn last_shengji_score_capture(&self) -> Option<&ShengjiScoreCaptureEffect> {
        self.games.shengji.score_capture.value.as_ref()
    }

    pub fn shengji_score_capture_serial(&self) -> u64 {
        self.games.shengji.score_capture.serial
    }

    pub fn take_shengji_events(&mut self) -> Vec<ShengjiEvent> {
        self.games.shengji.events.take()
    }

    fn reset_shengji_tracking(&mut self) {
        self.games.shengji.collected_score_cards.clear();
        self.games.shengji.pending_trick_score_cards.clear();
        self.games.shengji.finished_event_pending_snapshot = false;
        self.games.shengji.score_capture.clear();
    }

    pub(super) fn apply_shengji_snapshot(&mut self, snapshot: ShengjiSnapshot) {
        if self.prepare_game_snapshot(snapshot.match_id, snapshot.host_port, snapshot.you) {
            self.games.shengji.events.clear();
            self.reset_shengji_tracking();
        } else {
            if self
                .shengji_game()
                .is_some_and(|previous| previous.hand_number != snapshot.hand_number)
            {
                self.reset_shengji_tracking();
            }
            let captured = if self.games.shengji.finished_event_pending_snapshot {
                self.games.shengji.finished_event_pending_snapshot = false;
                Vec::new()
            } else {
                self.shengji_game().map_or_else(Vec::new, |previous| {
                    completed_shengji_trick_score_cards(previous, &snapshot)
                })
            };
            for card in captured {
                if !self.games.shengji.collected_score_cards.contains(&card) {
                    self.games.shengji.collected_score_cards.push(card);
                }
            }
        }
        if let ShengjiPhaseView::Finished { result, .. } = &snapshot.phase {
            self.last_finished_match =
                Some((result.settlement_id, result.reference_changes.clone()));
        }
        self.rules = Some(GameRules::Shengji(snapshot.rules));
        self.store_game_snapshot(snapshot);
    }

    pub(super) fn apply_shengji_event(&mut self, event: ShengjiEvent) {
        match &event {
            ShengjiEvent::CardsPlayed { play, .. } => {
                for card in play
                    .play
                    .cards
                    .iter()
                    .copied()
                    .filter(|card| card.points() > 0)
                {
                    if !self.games.shengji.pending_trick_score_cards.contains(&card) {
                        self.games.shengji.pending_trick_score_cards.push(card);
                    }
                }
            }
            ShengjiEvent::TrickFinished {
                winner,
                collecting_score,
                ..
            } => {
                let collecting_side_won = self
                    .shengji_game()
                    .and_then(|game| game.dealer)
                    .is_some_and(|dealer| winner.0 % 2 != dealer.0 % 2);
                if collecting_side_won {
                    let cards = std::mem::take(&mut self.games.shengji.pending_trick_score_cards);
                    if !cards.is_empty() {
                        let score_before =
                            self.shengji_game().map_or(0, |game| game.collecting_score);
                        self.games
                            .shengji
                            .score_capture
                            .publish(ShengjiScoreCaptureEffect {
                                cards: cards.clone(),
                                score_before,
                                score_after: *collecting_score,
                            });
                    }
                    for card in cards {
                        if !self.games.shengji.collected_score_cards.contains(&card) {
                            self.games.shengji.collected_score_cards.push(card);
                        }
                    }
                } else {
                    self.games.shengji.pending_trick_score_cards.clear();
                }
                self.games.shengji.finished_event_pending_snapshot = true;
            }
            ShengjiEvent::RedealRequired => {
                self.games.shengji.pending_trick_score_cards.clear();
                self.games.shengji.finished_event_pending_snapshot = false;
            }
            _ => {}
        }
        self.games.shengji.events.push(event);
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
