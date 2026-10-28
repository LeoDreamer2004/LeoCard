use super::statistics::DoudizhuStatistics;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery, HostError, RoomSession};
use leocard_doudizhu::achievements::DoudizhuAchievementTracker;
use leocard_doudizhu::{Card, GameState, MatchState, Phase, RuleSet, auto_play_action, bot_action};
use leocard_protocol::{
    ClientMessage, MatchId, PlayerReferenceChange, Revision, RoomId, ServerEvent,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DoudizhuSession {
    pub(super) room: RoomSession,
    pub(super) rules: RuleSet,
    pub(super) deck: Option<Vec<Card>>,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) match_state: Option<MatchState>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) automatic_elapsed: Duration,
    pub(super) statistics: DoudizhuStatistics,
    pub(super) achievements: DoudizhuAchievementTracker,
}

impl DoudizhuSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: RuleSet,
        deck: Vec<Card>,
    ) -> Result<Self, HostError> {
        GameState::new(rules, deck.clone(), 0)?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                rules.mode.players(),
                rules.mode.players(),
            ),
            rules,
            deck: Some(deck),
            game: None,
            match_id: None,
            match_state: None,
            finished_reference_changes: None,
            automatic_elapsed: Duration::ZERO,
            statistics: DoudizhuStatistics::default(),
            achievements: DoudizhuAchievementTracker::default(),
        })
    }

    pub fn room_id(&self) -> RoomId {
        self.room.room_id
    }

    pub fn revision(&self) -> Revision {
        self.room.revision
    }

    pub fn is_closed(&self) -> bool {
        self.room.closed
    }

    pub fn is_current_connection(&self, connection: ConnectionId) -> bool {
        self.room.player_id(connection).is_some()
    }

    pub fn heartbeat(&self) -> Vec<Delivery> {
        self.room.heartbeat()
    }

    pub fn handle(&mut self, connection: ConnectionId, message: ClientMessage) -> Vec<Delivery> {
        dispatch_client_command(self, connection, message)
    }

    pub(super) fn reset_automatic_delay(&mut self) {
        self.automatic_elapsed = Duration::ZERO;
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        let Some(game) = &self.game else {
            return Vec::new();
        };
        if elapsed.is_zero() {
            return Vec::new();
        }
        if game.phase == Phase::Finished {
            if !self.match_state.as_ref().expect("active match").complete() && self.everyone_ready()
            {
                self.automatic_elapsed += elapsed;
                if self.automatic_elapsed >= Duration::from_secs(3) {
                    self.continue_match();
                    self.room.bump_revision();
                    return self.broadcast_game(None);
                }
            }
            return Vec::new();
        }
        let participant = &self.room.players[usize::from(game.current)];
        let automatic = participant.is_bot
            || participant.auto_play
            || !participant.connected
            || participant.left;
        if !automatic {
            self.automatic_elapsed = Duration::ZERO;
            return Vec::new();
        }
        self.automatic_elapsed += elapsed;
        if self.automatic_elapsed >= Duration::from_millis(800) {
            let action = if participant.is_bot {
                bot_action(game)
            } else {
                auto_play_action(game)
            };
            let Some(action) = action else {
                return Vec::new();
            };
            let player = game.current;
            return self.apply_action(player, action, None);
        }
        Vec::new()
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        if self.room.disconnect_spectator(connection) {
            return Vec::new();
        }
        let Some(player) = self
            .room
            .players
            .iter_mut()
            .find(|p| p.connection == connection && p.connected)
        else {
            return Vec::new();
        };
        player.connected = false;
        if self.game.is_none() {
            player.left = true;
            player.seat = None;
            player.ready = false;
        }
        self.room.bump_revision();
        if self.room.host_connection == Some(connection) {
            self.room.closed = true;
            self.room.broadcast_event(None, ServerEvent::RoomClosed)
        } else if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        }
    }
}
