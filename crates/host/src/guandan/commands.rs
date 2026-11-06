use super::GuandanSession;
use crate::features::GameCommandHandler;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_guandan::{GuandanPhase, build_deck};
use leocard_protocol::{
    GameCommand, GameKind, GameViolation, GuandanCommand, PlayerViolation, RejectReason, RequestId,
    RoomViolation,
};
use std::time::Duration;

impl GameCommandHandler for GuandanSession {
    type Command = GuandanCommand;

    fn parse_command(command: GameCommand) -> Result<Self::Command, GameKind> {
        let GameCommand::Guandan(command) = command else {
            return Err(command.kind());
        };
        Ok(command)
    }

    fn handle_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: Self::Command,
    ) -> Vec<Delivery> {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let reason = match command {
            GuandanCommand::UpdateRules { rules } => {
                if self.game.is_some() {
                    RejectReason::Game(GameViolation::GameAlreadyStarted)
                } else if self.room.host_connection != Some(connection) {
                    RejectReason::Room(RoomViolation::OnlyHostCanConfigure)
                } else if !rules.is_valid() {
                    RejectReason::Game(GameViolation::InvalidRuleConfiguration)
                } else {
                    if rules != self.rules {
                        self.rules = rules;
                        self.room.reset_ready_after_rules_change();
                        let mut deck = build_deck();
                        fastrand::shuffle(&mut deck);
                        self.deck = Some(deck);
                        self.room.bump_revision();
                    }
                    return self.broadcast_lobby(Some((connection, request)));
                }
            }
            GuandanCommand::SetAutoPlay { enabled } => {
                if self
                    .game
                    .as_ref()
                    .is_none_or(|game| matches!(game.phase(), GuandanPhase::Finished(_)))
                {
                    RejectReason::Game(GameViolation::GameNotStarted)
                } else {
                    self.room.players[player.index()].auto_play = enabled;
                    self.automatic_elapsed = Duration::ZERO;
                    self.room.bump_revision();
                    return self.broadcast_game(Some((connection, request)));
                }
            }
            GuandanCommand::Act(action) => {
                if self.game.is_none() {
                    RejectReason::Game(GameViolation::GameNotStarted)
                } else {
                    return self.apply_action(player, action, Some((connection, request)));
                }
            }
        };
        self.room.reject(connection, request, reason)
    }
}
