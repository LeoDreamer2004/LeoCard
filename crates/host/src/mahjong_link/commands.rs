use super::MahjongLinkSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_mahjong_link::{MahjongLinkAction, MahjongLinkError};
use leocard_protocol::{
    GameViolation, MahjongLinkCommand, MahjongLinkEvent, RejectReason, RequestId,
};
use std::time::Duration;

impl MahjongLinkSession {
    pub(super) fn command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: MahjongLinkCommand,
    ) -> Vec<Delivery> {
        match self.apply_command(connection, command) {
            Ok(mut deliveries) => {
                deliveries.extend(if self.game.is_some() {
                    self.broadcast_game(Some((connection, request)))
                } else {
                    self.broadcast_lobby(Some((connection, request)))
                });
                deliveries
            }
            Err(reason) => self.reject(connection, request, reason),
        }
    }

    fn apply_command(
        &mut self,
        connection: ConnectionId,
        command: MahjongLinkCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self.require_player(connection)?;
        if self.game.is_none() {
            return Err(RejectReason::Game(GameViolation::GameNotStarted));
        }
        match command {
            MahjongLinkCommand::SetAutoPlay { enabled } => {
                let participant = self
                    .room
                    .players
                    .iter_mut()
                    .find(|participant| participant.id == player)
                    .expect("joined player");
                participant.auto_play = enabled;
                if self.game.as_ref().is_some_and(|game| game.complete()) {
                    participant.ready = enabled;
                }
                self.room.bump_revision();
                Ok(Vec::new())
            }
            MahjongLinkCommand::Act { action } => self.apply_action(player, action),
        }
    }

    pub(super) fn apply_action(
        &mut self,
        player: PlayerId,
        action: MahjongLinkAction,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let index = self
            .order
            .iter()
            .position(|id| *id == player)
            .ok_or(RejectReason::Game(MahjongLinkError::InvalidPlayer.into()))?;
        let game = self.game.as_mut().expect("active game");
        let outcome = game
            .act(index as u8, action)
            .map_err(|error| RejectReason::Game(error.into()))?;
        let event = MahjongLinkEvent {
            player,
            turn: game.turn(),
            outcome,
        };
        if game.complete() {
            self.room.prepare_rematch();
        }
        self.elapsed = Duration::ZERO;
        self.last_action = Some(event.clone());
        self.room.bump_revision();
        Ok(self
            .room
            .broadcast_game_events([event], self.match_id.map(|id| (id, None))))
    }
}
