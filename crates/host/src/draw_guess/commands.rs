use super::DrawGuessSession;
use crate::features::GameCommandHandler;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery};
use leocard_draw_guess::DrawGuessError;
use leocard_protocol::{
    DrawGuessCommand, DrawGuessEvent, GameCommand, GameKind, GameViolation, RejectReason,
    RequestId, RoomViolation, ServerEvent,
};

impl DrawGuessSession {
    fn apply_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: DrawGuessCommand,
    ) -> Result<Vec<Delivery>, RejectReason> {
        if matches!(
            &command,
            DrawGuessCommand::SelectLibrary { .. }
                | DrawGuessCommand::UploadLibrary { .. }
                | DrawGuessCommand::ResolveLibraryUpload { .. }
        ) {
            return self.handle_library_command(connection, request, command);
        }
        let player = self.require_player(connection)?;
        if let DrawGuessCommand::UpdateRules { rules } = command {
            if self.game.is_some() {
                return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
            }
            if self.room.host_connection != Some(connection) {
                return Err(RejectReason::Room(RoomViolation::OnlyHostCanConfigure));
            }
            self.rules = rules
                .validate()
                .map_err(|error| RejectReason::Game(error.into()))?;
            self.room.reset_ready_after_rules_change();
            self.room.bump_revision();
            return Ok(self.broadcast_lobby(Some((connection, request))));
        }
        let game = self
            .game
            .as_mut()
            .ok_or(RejectReason::Game(GameViolation::GameNotStarted))?;
        let turn = match &command {
            DrawGuessCommand::ChooseWord { turn, .. }
            | DrawGuessCommand::Draw { turn, .. }
            | DrawGuessCommand::Guess { turn, .. } => *turn,
            _ => unreachable!("lobby commands already handled"),
        };
        if turn != game.turn() {
            return Err(RejectReason::Game(DrawGuessError::WrongPhase.into()));
        }
        let event = match command {
            DrawGuessCommand::ChooseWord { choice, .. } => {
                game.choose_word(player, choice)
                    .map_err(|error| RejectReason::Game(error.into()))?;
                None
            }
            DrawGuessCommand::Guess { text, .. } => {
                game.guess(player, &text)
                    .map_err(|error| RejectReason::Game(error.into()))?;
                None
            }
            DrawGuessCommand::Draw { action, .. } => {
                game.draw(player, &action)
                    .map_err(|error| RejectReason::Game(error.into()))?;
                Some(DrawGuessEvent {
                    match_id: self.match_id.expect("live match"),
                    turn,
                    revision: game.canvas().revision(),
                    action,
                })
            }
            _ => unreachable!("lobby commands already handled"),
        };
        self.room.bump_revision();
        Ok(if let Some(event) = event {
            self.room.broadcast_event(
                Some((connection, request)),
                ServerEvent::GameEvent(event.into()),
            )
        } else {
            self.broadcast_game(Some((connection, request)))
        })
    }
}

impl GameCommandHandler for DrawGuessSession {
    type Command = DrawGuessCommand;

    fn parse_command(command: GameCommand) -> Result<Self::Command, GameKind> {
        let GameCommand::DrawGuess(command) = command else {
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
        match self.apply_command(connection, request, command) {
            Ok(deliveries) => deliveries,
            Err(reason) => self.reject(connection, request, reason),
        }
    }
}
