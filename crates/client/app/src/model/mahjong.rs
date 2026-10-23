use super::ClientModel;
use super::types::GameEventInbox;
use leocard_protocol::{GameRules, MahjongEvent, MahjongPhaseView, MahjongSnapshot};

#[derive(Clone, Debug, Default)]
pub(super) struct MahjongClientState {
    events: GameEventInbox<MahjongEvent>,
}

impl ClientModel {
    pub fn take_mahjong_events(&mut self) -> Vec<MahjongEvent> {
        self.games.mahjong.events.take()
    }

    pub(super) fn apply_mahjong_snapshot(&mut self, snapshot: MahjongSnapshot) {
        if self.prepare_game_snapshot(snapshot.match_id, snapshot.host_port, snapshot.you) {
            self.games.mahjong.events.clear();
        }
        if let MahjongPhaseView::Finished { result } = &snapshot.phase
            && result.match_complete
        {
            self.last_finished_match = Some((snapshot.match_id, result.reference_changes.clone()));
        }
        self.rules = Some(GameRules::Mahjong(snapshot.rules));
        self.store_game_snapshot(snapshot);
    }

    pub(super) fn apply_mahjong_event(&mut self, event: MahjongEvent) {
        self.games.mahjong.events.push(event);
    }
}
