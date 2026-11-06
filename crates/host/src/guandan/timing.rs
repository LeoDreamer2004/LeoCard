use super::GuandanSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::features::DecisionTiming;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_game_common::ActionTimeProfile;
use leocard_guandan::GuandanPhase;
use leocard_protocol::{ClientCommand, ClientMessage, GameCommand, GameSnapshot, GuandanCommand};

impl GuandanSession {
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
                .guandan
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

impl DecisionTiming for GuandanSession {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::Guandan(s)
                if !s.match_complete
                    && matches!(s.phase, GuandanPhase::Tribute | GuandanPhase::Playing) =>
            {
                // Current actor is public even during tribute. Never include private tribute cards
                // or recipient-only eligibility in the equality token or timing snapshot.
                PendingAction::for_players(
                    s.current,
                    (
                        "guandan",
                        s.match_id,
                        s.hand_number,
                        &s.phase,
                        s.played_cards.len(),
                        &s.table,
                        &s.last_actions,
                    ),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Guandan(command) = command else {
            return false;
        };
        matches!(command, GuandanCommand::Act(_))
    }
}
