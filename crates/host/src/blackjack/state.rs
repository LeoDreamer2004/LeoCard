use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery, HostError, RoomSession};
use leocard_blackjack::{BlackjackCard, BlackjackRuleSet, GameState, Phase};
use leocard_protocol::{
    ClientMessage, GameViolation, MatchId, PlayerId, PlayerReferenceChange, PlayerViolation,
    RejectReason, RequestId, Revision, RoomId, ServerEvent,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct BlackjackSession {
    pub(super) room: RoomSession,
    pub(super) rules: BlackjackRuleSet,
    pub(super) game: Option<GameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) order: Vec<PlayerId>,
    pub(super) initial_shoe: Option<Vec<BlackjackCard>>,
    pub(super) elapsed: Duration,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
}

impl BlackjackSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: BlackjackRuleSet,
        shoe: Vec<BlackjackCard>,
    ) -> Result<Self, HostError> {
        GameState::new(rules, 1, shoe.clone()).map_err(HostError::InvalidBlackjackGame)?;
        Ok(Self {
            room: RoomSession::new(
                room_id,
                host_port,
                usize::from(BlackjackRuleSet::MAX_PLAYERS),
            ),
            rules,
            game: None,
            match_id: None,
            order: Vec::new(),
            initial_shoe: Some(shoe),
            elapsed: Duration::ZERO,
            finished_reference_changes: None,
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

    pub(super) fn require_player(
        &self,
        connection: ConnectionId,
    ) -> Result<PlayerId, RejectReason> {
        self.room
            .player_id(connection)
            .ok_or(RejectReason::Player(PlayerViolation::NotJoined))
    }

    pub(super) fn require_finished(
        &self,
        connection: ConnectionId,
    ) -> Result<PlayerId, RejectReason> {
        let player = self.require_player(connection)?;
        let game = self
            .game
            .as_ref()
            .ok_or(RejectReason::Game(GameViolation::GameNotStarted))?;
        if game.phase() != Phase::Complete {
            return Err(RejectReason::Game(GameViolation::GameNotFinished));
        }
        Ok(player)
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        let Some(player) = self
            .room
            .players
            .iter_mut()
            .find(|player| player.connection == connection && !player.left)
        else {
            return Vec::new();
        };
        player.connected = false;
        if self.game.is_none() {
            player.left = true;
            player.seat = None;
        }
        self.room.bump_revision();
        if self.room.host_connection == Some(connection) {
            self.room.closed = true;
            return self.room.broadcast_event(None, ServerEvent::RoomClosed);
        }
        if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        }
    }

    pub(super) fn reject(
        &self,
        connection: ConnectionId,
        request: RequestId,
        reason: RejectReason,
    ) -> Vec<Delivery> {
        self.room.reject(connection, request, reason)
    }
}
