use super::BlackjackSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_blackjack::BlackjackPhase;
use leocard_protocol::{BlackjackCommand, ClientCommand, ClientMessage, GameCommand, GameSnapshot};

impl BlackjackSession {
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
                .blackjack
                .get_or_insert_with(Default::default)
                .action_time
                .record(millis);
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

    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::Blackjack(s)
                if !s.match_complete
                    && matches!(s.phase, BlackjackPhase::Betting | BlackjackPhase::Playing) =>
            {
                // `available` belongs to `you`; it is empty when viewing another actor's turn.
                let actor = s
                    .players
                    .iter()
                    .find(|p| Some(p.player.id) == s.current_player);
                let hand =
                    actor.and_then(|p| s.current_hand.and_then(|i| p.hands.get(usize::from(i))));
                let cards = (actor.map(|p| p.hands.len()), hand.map(|h| h.cards.len()));
                let state = (s.round, s.phase, s.current_hand, cards);
                PendingAction::for_players(
                    actor.map(|p| p.player.id),
                    ("blackjack", s.match_id, state),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Blackjack(command) = command else {
            return false;
        };
        matches!(command, BlackjackCommand::Act { .. })
    }
}
