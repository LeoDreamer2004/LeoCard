use super::DoudizhuSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::features::DecisionTiming;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_doudizhu::DoudizhuPhase;
use leocard_game_common::ActionTimeProfile;
use leocard_protocol::{ClientCommand, ClientMessage, DoudizhuCommand, GameCommand, GameSnapshot};

impl DoudizhuSession {
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
                .doudizhu
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

impl DecisionTiming for DoudizhuSession {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::Doudizhu(s)
                if !s.match_complete
                    && matches!(s.phase, DoudizhuPhase::Bidding | DoudizhuPhase::Playing) =>
            {
                let state = (
                    s.progress.hand_number,
                    s.phase,
                    &s.bids,
                    s.played.len(),
                    &s.table,
                );
                PendingAction::for_players(Some(s.current), ("doudizhu", s.match_id, state))
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Doudizhu(command) = command else {
            return false;
        };
        matches!(command, DoudizhuCommand::Act(_))
    }
}
