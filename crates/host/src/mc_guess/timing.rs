use super::McGuessSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::features::DecisionTiming;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_game_common::ActionTimeProfile;
use leocard_protocol::{ClientCommand, ClientMessage, GameCommand, GameSnapshot, McGuessCommand};

impl McGuessSession {
    pub(super) fn dispatch_with_action_timing(
        &mut self,
        connection: ConnectionId,
        message: ClientMessage,
    ) -> Vec<Delivery> {
        let is_decision = matches!(&message.command,
            ClientCommand::Game(command) if Self::is_decision(command));
        let timing = ActionTimingRequest::new(&self.room, connection, &message, is_decision);
        let deliveries = dispatch_client_command(self, connection, message);
        if let Some((profiles, millis)) = timing.complete(&mut self.room, &deliveries) {
            profiles
                .mc_guess
                .get_or_insert_with(Default::default)
                .record_action_time(millis);
        }
        self.refresh_action_timing();
        attach_action_timing(self, deliveries)
    }

    pub(super) fn refresh_action_timing(&mut self) {
        if self.room.closed || !self.game_started() {
            self.room.action_timing.clear();
            return;
        }
        let Some(recipient) = self.room.players.first().map(|player| player.id) else {
            self.room.action_timing.clear();
            return;
        };
        let snapshot = HostedGameLifecycle::game_snapshot(self, recipient);
        let windows = self.pending_actions(&snapshot);
        sync_action_timing(&mut self.room, Self::KIND, snapshot.match_id(), windows);
    }
}

impl DecisionTiming for McGuessSession {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::McGuess(s) => {
                s.players
                    .iter()
                    .filter(|p| s.round.can_guess(p.player.id, s.rules.mode))
                    .flat_map(|p| {
                        let player = p.player.id;
                        // Race guesses by other players do not create a new window for this player.
                        let guesses = s
                            .round
                            .guesses
                            .iter()
                            .filter(|g| g.player == player)
                            .count();
                        // Selecting nodes is not a guess and must not restart the clock.
                        let graph = s.graph.as_ref().map(|g| {
                            g.guesses
                                .iter()
                                .filter(|r| r.guess.player == player)
                                .count()
                        });
                        PendingAction::for_players(
                            Some(player),
                            (
                                "mc_guess",
                                s.match_id,
                                s.progress.game_index,
                                guesses,
                                graph,
                                s.round
                                    .skipped_players
                                    .iter()
                                    .filter(|p| **p == player)
                                    .count(),
                            ),
                        )
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::McGuess(command) = command else {
            return false;
        };
        matches!(command, McGuessCommand::Guess { .. } | McGuessCommand::Skip)
    }
}
