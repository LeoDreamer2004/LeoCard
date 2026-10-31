use super::McGuessSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    GameViolation, McGuessCommand, McGuessViolation, RejectReason, RequestId, RoomViolation,
};

impl McGuessSession {
    pub(super) fn command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: McGuessCommand,
    ) -> Vec<Delivery> {
        if let McGuessCommand::ReportMcProcess { running } = command {
            return self.report_mc_process(connection, request, running);
        }
        let result = self.apply_command(connection, command);
        match result {
            Ok(mut deliveries) if self.game.is_some() => {
                deliveries.extend(self.broadcast_game(Some((connection, request))));
                deliveries
            }
            Ok(_) => self.broadcast_lobby(Some((connection, request))),
            Err(RejectReason::Game(GameViolation::McGuess(
                McGuessViolation::NodeSwitchCoolingDown,
            ))) => Vec::new(),
            Err(reason) => self.reject(connection, request, reason),
        }
    }

    fn apply_command(
        &mut self,
        connection: ConnectionId,
        command: McGuessCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        let player = self.require_player(connection)?;
        if let McGuessCommand::UpdateRules { rules } = command {
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
        if matches!(command, McGuessCommand::DeveloperActivateAll)
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
        if let McGuessCommand::SelectNode { number } = command {
            game.select_node(player, number)
                .map_err(|error| RejectReason::Game(error.into()))?;
            self.bump_revision();
            return Ok(Vec::new());
        }
        match command {
            McGuessCommand::SelectNode { .. } => unreachable!("handled separately"),
            McGuessCommand::Guess { entry } => game.guess(player, &entry),
            McGuessCommand::Skip => game.skip(player),
            McGuessCommand::DeveloperActivateAll => game.reveal_graph(),
            McGuessCommand::UpdateRules { .. } => unreachable!("handled above"),
            McGuessCommand::ReportMcProcess { .. } => {
                unreachable!("handled separately from game actions")
            }
        }
        .map_err(|error| RejectReason::Game(error.into()))?;
        Ok(self.after_action())
    }
}
