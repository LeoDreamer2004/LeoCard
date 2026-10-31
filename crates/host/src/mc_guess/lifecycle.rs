use super::McGuessSession;
use crate::lifecycle::HostedGameLifecycle;
use crate::{ConnectionId, Delivery, RoomSession};
use leocard_game_common::PlayerId;
use leocard_mc_guess::McGuessRuleSet;
use leocard_protocol::{
    GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, McGuessSnapshot, PlayerScore,
    RejectReason, RequestId, RoomViolation, ServerEvent,
};

impl HostedGameLifecycle for McGuessSession {
    const KIND: GameKind = GameKind::McGuess;

    fn room(&self) -> &RoomSession {
        &self.room
    }

    fn room_mut(&mut self) -> &mut RoomSession {
        &mut self.room
    }

    fn game_started(&self) -> bool {
        self.game.is_some()
    }

    fn capacity(&self) -> u8 {
        McGuessRuleSet::MAX_PLAYERS
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot {
        let state = self.match_state.as_ref().expect("active match");
        McGuessSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.host_port,
            you: recipient,
            host: self.host_player_id().expect("room host"),
            players: state
                .scores()
                .map(|(id, _)| {
                    self.players
                        .iter()
                        .find(|player| player.id == id)
                        .expect("match participant belongs to its room")
                })
                .filter(|player| !player.left)
                .map(|player| player.public_metadata().into())
                .collect(),
            rules: self.rules,
            progress: state.progress(),
            round: self.game.as_ref().expect("active game").view(),
            graph: self.game.as_ref().expect("active game").graph_view(),
            guess_seconds_remaining: self.guess_seconds_remaining(),
            scores: state
                .scores()
                .map(|(player, score)| PlayerScore { player, score })
                .collect(),
            score_gains: state.gains().to_vec(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
            mc_process_players: self.process_presence.players(),
        }
        .into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::McGuess(command) = command else {
            return Err(command.kind());
        };
        Ok(self.command(connection, request, command))
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        let validation = self
            .room
            .validate_start(connection, McGuessRuleSet::MIN_PLAYERS, self.game.is_some())
            .and_then(|()| {
                if self.rules.is_valid() {
                    Ok(())
                } else {
                    Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration))
                }
            });
        if let Err(reason) = validation {
            return self.reject(connection, request, reason);
        }
        if let Err(error) = self.begin_match() {
            return self.reject(connection, request, RejectReason::Game(error.into()));
        }
        self.broadcast_game(Some((connection, request)))
    }

    fn return_to_lobby(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.require_finished(connection) {
            return self.reject(connection, request, reason);
        }
        if self.host_connection != Some(connection) {
            return self.reject(
                connection,
                request,
                RejectReason::Room(RoomViolation::OnlyHostCanReturnToLobby),
            );
        }
        self.game = None;
        self.match_id = None;
        self.match_state = None;
        self.finished_reference_changes = None;
        self.process_presence.clear();
        self.room.prepare_lobby();
        self.bump_revision();
        self.broadcast_lobby(Some((connection, request)))
    }

    fn play_again(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        if let Err(reason) = self.require_finished(connection) {
            return self.reject(connection, request, reason);
        }
        let player = self.player_id(connection).expect("validated participant");
        self.room.players[usize::from(player.0)].ready = true;
        self.bump_revision();
        let players: Vec<_> = self.players.iter().filter(|player| !player.left).collect();
        let continuing = !self
            .match_state
            .as_ref()
            .expect("active match")
            .progress()
            .is_last_game();
        if !players.is_empty()
            && (continuing || players.len() >= 2)
            && players.iter().all(|player| player.ready)
            && let Err(error) = self.continue_match()
        {
            return self.reject(connection, request, RejectReason::Game(error.into()));
        }
        self.broadcast_game(Some((connection, request)))
    }

    fn leave_room(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        let Ok(player) = self.require_player(connection) else {
            return self.snapshot(connection, request);
        };
        if self.host_connection == Some(connection) {
            return self.close_room(connection, request);
        }
        let participant = &mut self.room.players[usize::from(player.0)];
        let name = participant.name.clone();
        participant.connected = false;
        participant.left = true;
        self.process_presence.remove(player);
        self.bump_revision();
        let mut deliveries = vec![self.delivery(connection, Some(request), ServerEvent::LeftRoom)];
        deliveries.extend(
            self.room
                .broadcast_event(None, ServerEvent::PlayerLeft { name }),
        );
        deliveries.extend(if self.game.is_some() {
            self.broadcast_game(None)
        } else {
            self.broadcast_lobby(None)
        });
        deliveries
    }
}

impl McGuessSession {
    fn require_finished(&self, connection: ConnectionId) -> Result<(), RejectReason> {
        self.require_player(connection)?;
        let game = self
            .game
            .as_ref()
            .ok_or(RejectReason::Game(GameViolation::GameNotStarted))?;
        if !game.finished() {
            return Err(RejectReason::Game(GameViolation::GameNotFinished));
        }
        Ok(())
    }
}
