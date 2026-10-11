use super::MinecraftGuessSession;
use crate::{ConnectionId, Delivery, lifecycle::HostedGameLifecycle};
use leocard_protocol::{
    GameViolation, MinecraftGuessCommand, RejectReason, RequestId, RoomViolation,
};

impl MinecraftGuessSession {
    pub(super) fn command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: MinecraftGuessCommand,
    ) -> Vec<Delivery> {
        let result = self.apply_command(connection, command);
        match result {
            Ok(mut deliveries) if self.game.is_some() => {
                deliveries.extend(self.broadcast_game(Some((connection, request))));
                deliveries
            }
            Ok(_) => self.broadcast_lobby(Some((connection, request))),
            Err(reason) => self.reject(connection, request, reason),
        }
    }

    fn apply_command(
        &mut self,
        connection: ConnectionId,
        command: MinecraftGuessCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self.require_player(connection)?;
        if let MinecraftGuessCommand::UpdateRules { rules } = command {
            if self.game.is_some() {
                return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
            }
            if self.host_connection != Some(connection) {
                return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
            }
            if !rules.is_valid() {
                return Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration));
            }
            self.rules = rules;
            self.room.reset_ready_after_rules_change();
            self.bump_revision();
            return Ok(Vec::new());
        }
        let game = self
            .game
            .as_mut()
            .ok_or(RejectReason::Game(GameViolation::GameNotStarted))?;
        match command {
            MinecraftGuessCommand::Guess { entry } => game.guess(player.0, &entry),
            MinecraftGuessCommand::Skip => game.skip(player.0),
            MinecraftGuessCommand::UpdateRules { .. } => unreachable!("handled above"),
        }
        .map_err(|error| RejectReason::Game(GameViolation::MinecraftGuess(error)))?;
        Ok(self.after_action())
    }
}
