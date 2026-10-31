use super::RoomSession;
use leocard_game_common::{PlayerId, PlayerPersona};
use leocard_protocol::LobbyEntranceView;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub(super) struct RoomEntrance {
    sequence: u64,
    player: PlayerId,
    started: Instant,
}

impl RoomSession {
    pub(super) fn cancel_entrance_for(&mut self, player: PlayerId) {
        if self
            .entrance
            .as_ref()
            .is_some_and(|entrance| entrance.player == player)
        {
            self.entrance = None;
        }
    }

    pub(super) fn welcome_new_player(&mut self, player: PlayerId) {
        let participant = &self.players[player.index()];
        if self.host_connection != Some(participant.connection)
            && PlayerPersona::from(participant.name.as_str()) == PlayerPersona::Yang120
        {
            self.entrance = Some(RoomEntrance {
                sequence: self.revision.0,
                player,
                started: Instant::now(),
            });
        }
    }

    pub(crate) fn lobby_entrance(&self) -> Option<LobbyEntranceView> {
        let entrance = self.entrance.as_ref()?;
        let elapsed = entrance.started.elapsed();
        if elapsed >= Duration::from_millis(u64::from(LobbyEntranceView::DURATION_MILLIS))
            || !self
                .players
                .iter()
                .any(|player| player.id == entrance.player && player.connected && !player.left)
        {
            return None;
        }
        Some(LobbyEntranceView {
            sequence: entrance.sequence,
            player: entrance.player,
            elapsed_millis: elapsed.as_millis() as u16,
        })
    }
}
