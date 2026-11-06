use super::TexasSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::features::DecisionTiming;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_game_common::ActionTimeProfile;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, TexasCommand, TexasPhaseView,
};

impl TexasSession {
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
                .texas
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

impl DecisionTiming for TexasSession {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::Texas(s) if matches!(s.phase, TexasPhaseView::ChoosingRunout) => {
                PendingAction::for_players(
                    s.current_player,
                    ("texas-runout", s.match_id, s.hand_number),
                )
            }
            GameSnapshot::Texas(s)
                if matches!(s.phase, TexasPhaseView::Betting { .. })
                    && s.blind_to_post.is_none() =>
            {
                let actor = s
                    .players
                    .iter()
                    .find(|p| Some(p.player.id) == s.current_player && !p.folded && !p.all_in);
                let bets = (
                    s.current_bet,
                    s.minimum_raise_to,
                    actor.map(|p| p.committed_street),
                );
                PendingAction::for_players(
                    actor.map(|p| p.player.id),
                    ("texas", s.match_id, s.hand_number, &s.phase, bets),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Texas(command) = command else {
            return false;
        };
        matches!(
            command,
            TexasCommand::Act { .. } | TexasCommand::ChooseRunout { .. }
        )
    }
}
