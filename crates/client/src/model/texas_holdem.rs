use super::ClientModel;
use super::types::GameEventInbox;
use leocard_protocol::{TexasHoldemEvent, TexasHoldemPhaseView, TexasHoldemSnapshot};

#[derive(Clone, Debug, Default)]
pub(super) struct TexasHoldemClientState {
    events: GameEventInbox<TexasHoldemEvent>,
}

impl ClientModel {
    pub fn take_texas_holdem_events(&mut self) -> Vec<TexasHoldemEvent> {
        self.games.texas_holdem.events.take()
    }

    pub(super) fn apply_texas_holdem_snapshot(&mut self, snapshot: TexasHoldemSnapshot) {
        if self.prepare_game_snapshot(snapshot.match_id, snapshot.host_port, snapshot.you) {
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
        self.store_game_snapshot(snapshot);
    }

    pub(super) fn apply_texas_holdem_event(&mut self, event: TexasHoldemEvent) {
        self.games.texas_holdem.events.push(event);
    }
}
