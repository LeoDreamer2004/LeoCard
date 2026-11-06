use super::QiGui523Session;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, QiGui523Command, QiGui523PhaseView,
};

impl QiGui523Session {
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
                .qigui523
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
            GameSnapshot::QiGui523(s) if matches!(s.phase, QiGui523PhaseView::Playing) => {
                let Some(trick) = &s.trick else {
                    return Vec::new();
                };
                let actor = s
                    .players
                    .iter()
                    .find(|p| p.player.id == trick.current_player && p.hand_len > 0);
                let progress = (s.played_cards.len(), trick.leader, trick.records.len());
                PendingAction::for_players(
                    actor.map(|p| p.player.id),
                    ("qigui523", s.match_id, progress),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::QiGui523(command) = command else {
            return false;
        };
        matches!(
            command,
            QiGui523Command::PlayCards { .. } | QiGui523Command::Pass
        )
    }
}
