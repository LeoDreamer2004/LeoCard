use crate::action_timing::attach_action_timing;
use crate::features::GameAutomation;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, HostError, RoomSession, new_match_id};
use leocard_game_common::PlayerId;
use leocard_game_common::mahjong::MahjongTileKind;
use leocard_mahjong_link::{
    MahjongLinkBoard, MahjongLinkGameState, MahjongLinkRuleSet, build_deck,
};
use leocard_protocol::{
    ClientMessage, GameViolation, MahjongLinkEvent, MatchId, PlayerViolation, RejectReason,
    RequestId, Revision, RoomId, ServerEvent,
};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct MahjongLinkSession {
    pub(super) room: RoomSession,
    pub(super) rules: MahjongLinkRuleSet,
    pub(super) game: Option<MahjongLinkGameState>,
    pub(super) match_id: Option<MatchId>,
    pub(super) order: Vec<PlayerId>,
    pub(super) initial_deck: Option<Vec<MahjongTileKind>>,
    pub(super) elapsed: Duration,
    pub(super) last_action: Option<MahjongLinkEvent>,
}

impl MahjongLinkSession {
    pub fn new(
        room_id: RoomId,
        host_port: u16,
        rules: MahjongLinkRuleSet,
        deck: Vec<MahjongTileKind>,
    ) -> Result<Self, HostError> {
        MahjongLinkBoard::new(deck.clone())?;
        Ok(Self {
            room: RoomSession::new_with_seat_count(
                room_id,
                host_port,
                MahjongLinkRuleSet::MAX_PLAYERS,
                MahjongLinkRuleSet::MAX_PLAYERS,
            ),
            rules,
            game: None,
            match_id: None,
            order: Vec::new(),
            initial_deck: Some(deck),
            elapsed: Duration::ZERO,
            last_action: None,
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
        self.dispatch_with_action_timing(connection, message)
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
        if !game.complete() {
            return Err(RejectReason::Game(GameViolation::GameNotFinished));
        }
        Ok(player)
    }

    pub(super) fn begin_match(&mut self) {
        self.room.remove_departed_players();
        self.order = self.room.players.iter().map(|player| player.id).collect();
        self.match_id = Some(new_match_id());
        let deck = self.initial_deck.take().unwrap_or_else(|| {
            let mut deck = build_deck();
            fastrand::shuffle(&mut deck);
            deck
        });
        self.game =
            Some(MahjongLinkGameState::new(self.order.len() as u8, deck).expect("validated deck"));
        self.last_action = None;
        self.elapsed = Duration::ZERO;
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
        active.len() >= usize::from(MahjongLinkRuleSet::MIN_PLAYERS)
            && active.iter().all(|player| player.ready)
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        self.room.action_timing.advance(elapsed);
        let deliveries = self.advance_game_time(elapsed);
        self.refresh_action_timing();
        attach_action_timing(self, deliveries)
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

impl GameAutomation for MahjongLinkSession {
    fn advance_game_time(&mut self, elapsed: Duration) -> Vec<Delivery> {
        if elapsed.is_zero() || self.room.closed {
            return Vec::new();
        }
        let Some(game) = &self.game else {
            return Vec::new();
        };
        if game.complete() {
            if !self.everyone_ready() {
                self.elapsed = Duration::ZERO;
                return Vec::new();
            }
            self.elapsed += elapsed;
            if self.elapsed >= Duration::from_secs(3) {
                self.begin_match();
                return self.broadcast_game(None);
            }
            return Vec::new();
        }
        let player = self.order[usize::from(game.current_player().expect("live turn"))];
        let participant = self
            .room
            .players
            .iter()
            .find(|participant| participant.id == player)
            .expect("match participant");
        if !(participant.is_bot
            || participant.auto_play
            || !participant.connected
            || participant.left)
        {
            self.elapsed = Duration::ZERO;
            return Vec::new();
        }
        self.elapsed += elapsed;
        if self.elapsed < Duration::from_millis(1500) {
            return Vec::new();
        }
        let action = game.automatic_action();
        let mut deliveries = self
            .apply_action(player, action)
            .expect("legal automatic action");
        deliveries.extend(self.broadcast_game(None));
        deliveries
    }
}
