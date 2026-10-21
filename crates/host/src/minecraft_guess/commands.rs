use super::MinecraftGuessSession;
use crate::{ConnectionId, Delivery, lifecycle::HostedGameLifecycle};
use leocard_minecraft_guess::GameError;
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
        if let MinecraftGuessCommand::ReportMinecraftProcess { running } = command {
            return self.report_minecraft_process(connection, request, running);
        }
        let result = self.apply_command(connection, command);
        match result {
            Ok(mut deliveries) if self.game.is_some() => {
                deliveries.extend(self.broadcast_game(Some((connection, request))));
                deliveries
            }
            Ok(_) => self.broadcast_lobby(Some((connection, request))),
            Err(RejectReason::Game(GameViolation::MinecraftGuess(
                GameError::NodeSwitchCoolingDown,
            ))) => Vec::new(),
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
        if matches!(command, MinecraftGuessCommand::DeveloperActivateAll)
            && !(cfg!(debug_assertions) || cfg!(feature = "developer"))
        {
            return Err(RejectReason::Game(
                GameViolation::DeveloperFeatureUnavailable,
            ));
        }
        let game = self
            .game
            .as_mut()
            .ok_or(RejectReason::Game(GameViolation::GameNotStarted))?;
        if let MinecraftGuessCommand::SelectNode { number } = command {
            game.select_node(player.0, number)
                .map_err(|error| RejectReason::Game(GameViolation::MinecraftGuess(error)))?;
            self.bump_revision();
            return Ok(Vec::new());
        }
        match command {
            MinecraftGuessCommand::SelectNode { .. } => unreachable!("handled separately"),
            MinecraftGuessCommand::Guess { entry } => game.guess(player.0, &entry),
            MinecraftGuessCommand::Skip => game.skip(player.0),
            MinecraftGuessCommand::DeveloperActivateAll => game.reveal_graph(),
            MinecraftGuessCommand::UpdateRules { .. } => unreachable!("handled above"),
            MinecraftGuessCommand::ReportMinecraftProcess { .. } => {
                unreachable!("handled separately from game actions")
            }
        }
        .map_err(|error| RejectReason::Game(GameViolation::MinecraftGuess(error)))?;
        Ok(self.after_action())
    }
}
