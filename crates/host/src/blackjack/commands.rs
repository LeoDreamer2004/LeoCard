use super::BlackjackSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_blackjack::{BlackjackAction, Phase};
use leocard_protocol::{
    BlackjackCommand, BlackjackEvent, GameViolation, PlayerId, RejectReason, RequestId,
    RoomViolation,
};
use std::time::Duration;

impl BlackjackSession {
    pub(super) fn command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: BlackjackCommand,
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
        command: BlackjackCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self.require_player(connection)?;
        if let BlackjackCommand::UpdateRules { rules } = command {
            if self.game.is_some() {
                return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
            }
            if self.room.host_connection != Some(connection) {
                return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
            }
            if !rules.is_valid() {
                return Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration));
            }
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            self.room.bump_revision();
            return Ok(Vec::new());
        }
        if self.game.is_none() {
            return Err(RejectReason::Game(GameViolation::GameNotStarted));
        }
        match command {
            BlackjackCommand::SetAutoPlay { enabled } => {
                let participant = self
                    .room
                    .players
                    .iter_mut()
                    .find(|participant| participant.id == player)
                    .expect("joined player");
                participant.auto_play = enabled;
                if self
                    .game
                    .as_ref()
                    .is_some_and(|game| game.phase() == Phase::Complete)
                {
                    participant.ready = enabled;
                }
                self.room.bump_revision();
                Ok(Vec::new())
            }
            BlackjackCommand::Act { action } => self.apply_action(player, action),
            BlackjackCommand::UpdateRules { .. } => unreachable!("handled above"),
        }
    }

    pub(super) fn apply_action(
        &mut self,
        player: PlayerId,
        action: BlackjackAction,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let index = self
            .order
            .iter()
            .position(|id| *id == player)
            .expect("active match player");
        let game = self.game.as_mut().expect("active game");
        game.act(index, action)
            .map_err(|error| RejectReason::Game(GameViolation::Blackjack(error)))?;
        let round = game.round();
        let finished = game.phase() == Phase::Complete;
        self.elapsed = Duration::ZERO;
        let mut events = vec![BlackjackEvent::ActionApplied { player, action }];
        if finished {
            self.settle_match();
            self.room.prepare_rematch();
            events.push(BlackjackEvent::RoundFinished { round });
        }
        self.room.bump_revision();
        Ok(self
            .room
            .broadcast_game_events(events, self.match_id.map(|id| (id, Some(u32::from(round))))))
    }
}
