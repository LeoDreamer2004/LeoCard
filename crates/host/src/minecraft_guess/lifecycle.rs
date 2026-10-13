use super::MinecraftGuessSession;
use crate::{ConnectionId, Delivery, RoomSession, lifecycle::HostedGameLifecycle};
use leocard_minecraft_guess::MinecraftGuessRuleSet;
use leocard_protocol::{
    GameCommand, GameKind, GameRules, GameSnapshot, GameViolation, MinecraftGuessSnapshot,
    PlayerId, PlayerInteractionKind, PlayerScore, PlayerViolation, RejectReason, RequestId,
    RoomViolation, ServerEvent,
};

impl HostedGameLifecycle for MinecraftGuessSession {
    const KIND: GameKind = GameKind::MinecraftGuess;

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
        MinecraftGuessRuleSet::MAX_PLAYERS
    }

    fn game_rules(&self) -> GameRules {
        self.rules.into()
    }

    fn game_snapshot(&self, recipient: PlayerId) -> GameSnapshot {
        MinecraftGuessSnapshot {
            match_id: self.match_id.expect("active match"),
            host_port: self.host_port,
            you: recipient,
            host: self.host_player_id().expect("room host"),
            players: self
                .players
                .iter()
                .filter(|player| !player.left)
                .map(|player| player.public_metadata().into())
                .collect(),
            rules: self.rules,
            progress: self.match_state.as_ref().expect("active match").progress(),
            round: self.game.as_ref().expect("active game").view(),
            guess_seconds_remaining: self.guess_seconds_remaining(),
            scores: self
                .match_state
                .as_ref()
                .expect("active match")
                .scores()
                .map(|(player, score)| PlayerScore {
                    player: PlayerId(player),
                    score,
                })
                .collect(),
            score_gains: self
                .match_state
                .as_ref()
                .expect("active match")
                .gains()
                .to_vec(),
            reference_changes: self.finished_reference_changes.clone().unwrap_or_default(),
            minecraft_process_players: self.process_presence.players(),
        }
        .into()
    }

    fn handle_game_command(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        command: GameCommand,
    ) -> Result<Vec<Delivery>, GameKind> {
        let GameCommand::MinecraftGuess(command) = command else {
            return Err(command.kind());
        };
        Ok(self.command(connection, request, command))
    }

    fn start_game(&mut self, connection: ConnectionId, request: RequestId) -> Vec<Delivery> {
        let validation = self.require_player(connection).and_then(|_| {
            if !self.rules.is_valid() {
                return Err(RejectReason::Game(GameViolation::InvalidRuleConfiguration));
            }
            if self.game.is_some() {
                return Err(RejectReason::Game(GameViolation::GameAlreadyStarted));
            }
            if self.host_connection != Some(connection) {
                return Err(RejectReason::Room(RoomViolation::OnlyHostCanStart));
            }
            let players: Vec<_> = self.players.iter().filter(|player| !player.left).collect();
            if players.len() < usize::from(MinecraftGuessRuleSet::MIN_PLAYERS) {
                return Err(RejectReason::Room(RoomViolation::NotEnoughPlayers {
                    minimum: MinecraftGuessRuleSet::MIN_PLAYERS,
                    actual: players.len() as u8,
                }));
            }
            if players.iter().any(|player| player.seat.is_none()) {
                return Err(RejectReason::Room(RoomViolation::MustSelectSeat));
            }
            let unready: Vec<_> = players
                .iter()
                .filter(|player| !player.ready || (!player.connected && !player.is_bot))
                .map(|player| player.id)
                .collect();
            if !unready.is_empty() {
                return Err(RejectReason::Room(RoomViolation::PlayersNotReady {
                    players: unready,
                }));
            }
            Ok(())
        });
        if let Err(reason) = validation {
            return self.reject(connection, request, reason);
        }
        self.begin_match();
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
        self.room.remove_bots();
        self.room.reset_ready_after_rules_change();
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
        {
            self.continue_match();
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

    fn interact(
        &mut self,
        connection: ConnectionId,
        request: RequestId,
        target: PlayerId,
        kind: PlayerInteractionKind,
    ) -> Vec<Delivery> {
        let result = self.require_player(connection).and_then(|source| {
            if self.game.is_none() {
                return Err(RejectReason::Game(GameViolation::GameNotStarted));
            }
            if source == target {
                return Err(RejectReason::Room(RoomViolation::CannotInteractWithSelf));
            }
            if !self
                .players
                .iter()
                .any(|player| player.id == target && !player.left)
            {
                return Err(RejectReason::Player(PlayerViolation::NotJoined));
            }
            Ok(self
                .room
                .record_interaction(source, target, kind, fastrand::u32(..)))
        });
        match result {
            Ok(interaction) => {
                self.bump_revision();
                self.room.broadcast_event(
                    Some((connection, request)),
                    ServerEvent::PlayerInteraction(interaction),
                )
            }
            Err(reason) => self.reject(connection, request, reason),
        }
    }
}

impl MinecraftGuessSession {
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
