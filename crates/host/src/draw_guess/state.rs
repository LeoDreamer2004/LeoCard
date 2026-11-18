use super::libraries::HostedWordLibraries;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery, HostError, RoomSession, new_match_id};
use leocard_draw_guess::{DrawGuessGame, DrawGuessPhase, DrawGuessRuleSet};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ClientMessage, GameViolation, MatchId, PlayerReferenceChange, PlayerViolation, RejectReason,
    RequestId, Revision, RoomId, ServerEvent,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct DrawGuessSession {
    pub(super) room: RoomSession,
    pub(super) rules: DrawGuessRuleSet,
    pub(super) libraries: HostedWordLibraries,
    pub(super) game: Option<DrawGuessGame>,
    pub(super) match_id: Option<MatchId>,
    pub(super) finished_reference_changes: Option<Vec<PlayerReferenceChange>>,
    pub(super) order: Vec<PlayerId>,
    pub(super) clock_remainder: Duration,
    pub(super) broadcast_elapsed: Duration,
}

impl DrawGuessSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: DrawGuessRuleSet,
    ) -> Result<Self, HostError> {
        let rules = rules.validate()?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                DrawGuessRuleSet::MAX_PLAYERS,
                DrawGuessRuleSet::MAX_PLAYERS,
            ),
            rules,
            libraries: HostedWordLibraries::default(),
            game: None,
            match_id: None,
            finished_reference_changes: None,
            order: Vec::new(),
            clock_remainder: Duration::ZERO,
            broadcast_elapsed: Duration::ZERO,
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
        if game.phase() != DrawGuessPhase::Finished {
            return Err(RejectReason::Game(GameViolation::GameNotFinished));
        }
        Ok(player)
    }

    pub(super) fn begin_match(&mut self) {
        for player in &mut self.room.players {
            if !player.connected && !player.is_bot {
                player.left = true;
            }
        }
        self.room.remove_departed_players();
        let mut players: Vec<_> = self
            .room
            .players
            .iter()
            .map(|player| (player.seat, player.id))
            .collect();
        players.sort_by_key(|(seat, _)| seat.map(|seat| seat.0));
        self.order = players.into_iter().map(|(_, id)| id).collect();
        self.libraries.offers.clear();
        let mut words = self.libraries.selected.words.clone();
        fastrand::shuffle(&mut words);
        self.game = Some(
            DrawGuessGame::new(self.rules, self.order.clone(), words)
                .expect("validated roster and built-in words"),
        );
        self.match_id = Some(new_match_id());
        self.finished_reference_changes = None;
        self.clock_remainder = Duration::ZERO;
        self.broadcast_elapsed = Duration::ZERO;
        for player in &mut self.room.players {
            player.ready = false;
        }
        self.room.bump_revision();
    }

    pub(super) fn everyone_ready(&self) -> bool {
        let active: Vec<_> = self
            .room
            .players
            .iter()
            .filter(|player| !player.left && (player.connected || player.is_bot))
            .collect();
        active.len() >= usize::from(DrawGuessRuleSet::MIN_PLAYERS)
            && active.iter().all(|player| player.ready)
    }

    pub fn disconnect(&mut self, connection: ConnectionId) -> Vec<Delivery> {
        if self.room.disconnect_spectator(connection) {
            return self.room.broadcast_moderation(None);
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
        if self.game.is_none() {
            player.left = true;
            player.seat = None;
        }
        if let Some(game) = &mut self.game {
            game.set_connected(player.id, false);
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
