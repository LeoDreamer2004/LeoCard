use super::*;
use leocard_protocol::{GameSnapshot, TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemSnapshot};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub(super) struct TexasHoldemClientState {
    events: VecDeque<TexasHoldemEvent>,
}

impl ClientModel {
    pub fn take_texas_holdem_events(&mut self) -> Vec<TexasHoldemEvent> {
        self.games.texas_holdem.events.drain(..).collect()
    }

    pub(super) fn apply_texas_holdem_snapshot(&mut self, snapshot: TexasHoldemSnapshot) {
        self.host_port = Some(snapshot.host_port);
        if self.active_match_id != Some(snapshot.match_id) {
            self.games.texas_holdem.events.clear();
        }
        if let TexasHoldemPhaseView::HandComplete {
            tournament_complete: true,
            reference_changes,
            ..
        } = &snapshot.phase
        {
            self.last_finished_match = Some((snapshot.match_id, reference_changes.clone()));
        }
        self.active_match_id = Some(snapshot.match_id);
        self.you = Some(snapshot.you);
        self.game = Some(GameSnapshot::TexasHoldem(snapshot));
        self.lobby = None;
        self.rejection.value = None;
    }

    pub(super) fn apply_texas_holdem_event(&mut self, event: TexasHoldemEvent) {
        self.games.texas_holdem.events.push_back(event);
    }
}
