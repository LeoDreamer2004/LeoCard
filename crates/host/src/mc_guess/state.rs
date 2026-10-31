use super::generation::GraphGeneration;
use super::{GuessClock, McProcessPresence};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_game_common::PlayerId;
use leocard_mc_guess::{McGuessGameState, McGuessMatchState, McGuessMode, McGuessRuleSet};
use leocard_protocol::{
    ClientMessage, MatchId, PlayerReferenceChange, PlayerViolation, RejectReason, Revision, RoomId,
    ServerEvent,
};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct McGuessSession {
    pub(super) room: RoomSession,
    pub(super) rules: McGuessRuleSet,
    pub(super) game: Option<McGuessGameState>,
    pub(super) generation: Option<GraphGeneration>,
    pub(super) match_id: Option<MatchId>,
    pub(super) match_state: Option<McGuessMatchState>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) clock: GuessClock,
    pub(super) process_presence: McProcessPresence,
}

impl Deref for McGuessSession {
    type Target = RoomSession;

    fn deref(&self) -> &Self::Target {
        &self.room
    }
}

impl DerefMut for McGuessSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.room
    }
}

impl McGuessSession {
    pub fn new(room_id: RoomId, host_port: u16, rules: McGuessRuleSet) -> Self {
        Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                McGuessRuleSet::MAX_PLAYERS,
                McGuessRuleSet::MAX_PLAYERS,
            ),
            rules,
            game: None,
            generation: None,
            match_id: None,
            match_state: None,
            clock: GuessClock::default(),
            finished_reference_changes: None,
            process_presence: McProcessPresence::default(),
        }
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

    pub(super) fn after_action(&mut self) -> Vec<Delivery> {
        if self.rules.mode != McGuessMode::Race {
            self.clock = GuessClock::default();
        }
        let gains = self.game.as_mut().expect("active game").take_score_gains();
        self.match_state
            .as_mut()
            .expect("active match")
            .record_gains(gains);
        let events = self.analyze_action();
        self.settle_match();
        if self.game.as_ref().is_some_and(|game| game.finished()) {
            self.room.prepare_rematch();
        }
        self.bump_revision();
        self.deliver_game_events(events)
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        if self.room.disconnect_spectator(connection) {
            return Vec::new();
        }
        let Some(player) = self
            .room
            .players
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
        else {
            return Vec::new();
        };
        player.connected = false;
        let player_id = player.id;
        self.process_presence.remove(player_id);
        if self.room.host_connection == Some(connection) {
            self.room.closed = true;
            self.bump_revision();
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
        }
        if self.game.is_none() {
            let player = self
                .room
                .players
                .iter_mut()
                .find(|player| player.connection == connection)
                .expect("known participant");
            player.left = true;
            player.seat = None;
        }
        self.bump_revision();
        if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub(super) fn require_player(
        &self,
        connection: ConnectionId,
    ) -> Result<PlayerId, RejectReason> {
        self.player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))
    }
}
