use super::ShengjiSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, ShengjiCommand, ShengjiPhaseView,
};
use leocard_shengji::ShengjiFiveTrumpCrossingStage;

impl ShengjiSession {
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
                .shengji
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
        let mut windows = snapshot_windows(snapshot);
        // Confirmation eligibility is private, so merge every recipient's view here.
        if let GameSnapshot::Shengji(game) = snapshot
            && matches!(
                game.phase,
                ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. }
            )
        {
            for player in &self.room.players {
                if player.id != game.you {
                    windows.extend(snapshot_windows(&HostedGameLifecycle::game_snapshot(
                        self, player.id,
                    )));
                }
            }
        }
        windows
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Shengji(command) = command else {
            return false;
        };
        !matches!(
            command,
            ShengjiCommand::SetAutoPlay { .. } | ShengjiCommand::UpdateRules { .. }
        )
    }
}

fn snapshot_windows(snapshot: &GameSnapshot) -> Vec<PendingAction> {
    match snapshot {
        GameSnapshot::Shengji(s) => {
            let stage = match &s.phase {
                // Optional declarations during dealing are not required actions.
                ShengjiPhaseView::Dealing { .. } => return Vec::new(),
                ShengjiPhaseView::BiddingGrace { power_outage, .. } => 1 + u8::from(*power_outage),
                ShengjiPhaseView::Burying => 3,
                ShengjiPhaseView::BottomCopying { .. } => 4,
                ShengjiPhaseView::BottomCopyBurying { .. } => 5,
                ShengjiPhaseView::FiveTrumpCrossing { stage, .. } => 6 + *stage as u8,
                ShengjiPhaseView::Playing if s.throw_failure.is_none() => 8,
                _ => return Vec::new(),
            };
            let players = s
                .players
                .iter()
                .map(|p| p.player.id)
                .filter(|p| match &s.phase {
                    ShengjiPhaseView::BiddingGrace { you_confirmed, .. } => {
                        *p == s.you && !*you_confirmed
                    }
                    ShengjiPhaseView::Burying => Some(*p) == s.dealer,
                    ShengjiPhaseView::BottomCopying { player, .. }
                    | ShengjiPhaseView::BottomCopyBurying { player } => p == player,
                    ShengjiPhaseView::FiveTrumpCrossing {
                        stage: ShengjiFiveTrumpCrossingStage::Deciding,
                        eligible,
                        decided,
                        ..
                    } => eligible.contains(p) && !decided.contains(p),
                    ShengjiPhaseView::FiveTrumpCrossing {
                        crossing, returned, ..
                    } => crossing.contains(&PlayerId((p.0 + 2) % 4)) && !returned.contains(p),
                    // Also None during completed-trick presentation pauses.
                    ShengjiPhaseView::Playing => Some(*p) == s.current_player,
                    _ => false,
                });
            let progress = (s.bottom_copy_decisions.len(), s.played_cards.len());
            let state = (s.hand_number, stage, &s.declaration, progress);
            PendingAction::for_players(players, ("shengji", s.match_id, state))
        }
        _ => Vec::new(),
    }
}
