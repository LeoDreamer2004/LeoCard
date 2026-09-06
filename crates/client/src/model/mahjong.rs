use std::collections::VecDeque;

use leocard_protocol::{GameRules, GameSnapshot, MahjongEvent, MahjongSnapshot};

use super::*;

#[derive(Clone, Debug, Default)]
pub(super) struct MahjongClientState {
    events: VecDeque<MahjongEvent>,
}

impl ClientModel {
    pub fn take_mahjong_events(&mut self) -> Vec<MahjongEvent> {
        self.games.mahjong.events.drain(..).collect()
    }

    pub(super) fn apply_mahjong_snapshot(&mut self, snapshot: MahjongSnapshot) {
        if self.active_match_id != Some(snapshot.match_id) {
            self.games.mahjong.events.clear();
        }
        self.host_port = Some(snapshot.host_port);
        self.active_match_id = Some(snapshot.match_id);
        self.you = Some(snapshot.you);
        self.rules = Some(GameRules::Mahjong(snapshot.rules));
        self.game = Some(GameSnapshot::Mahjong(snapshot));
        self.lobby = None;
        self.rejection.value = None;
    }

    pub(super) fn apply_mahjong_event(&mut self, event: MahjongEvent) {
        self.games.mahjong.events.push_back(event);
    }
}
