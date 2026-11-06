use super::MahjongSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::features::DecisionTiming;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_game_common::ActionTimeProfile;
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, MahjongCommand, MahjongPhaseView,
};

impl MahjongSession {
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
                .mahjong
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

impl DecisionTiming for MahjongSession {
    fn pending_actions(&self, snapshot: &GameSnapshot) -> Vec<PendingAction> {
        match snapshot {
            GameSnapshot::Mahjong(s) => {
                if matches!(s.phase, MahjongPhaseView::WaitingForClaims) {
                    let Some(claim) = &s.pending_claim else {
                        return Vec::new();
                    };
                    // Other responders leaving waiting_for must not restart this player's clock.
                    let source = (claim.source, claim.tile, claim.robbing_kong);
                    return PendingAction::for_players(
                        claim.waiting_for.iter().copied(),
                        ("mahjong-claim", s.match_id, s.discards.len(), source),
                    );
                }
                if !matches!(s.phase, MahjongPhaseView::Playing) {
                    return Vec::new();
                }
                let actor = s.players.iter().find(|p| p.player.id == s.current_player);
                let hand = actor.map(|p| (p.concealed_count, p.melds.len()));
                PendingAction::for_players(
                    actor.map(|p| p.player.id),
                    (
                        "mahjong-play",
                        s.match_id,
                        s.discards.len(),
                        s.wall_len,
                        hand,
                    ),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Mahjong(command) = command else {
            return false;
        };
        !matches!(
            command,
            MahjongCommand::SetAutoPlay { .. }
                | MahjongCommand::UpdateRules { .. }
                | MahjongCommand::SetDeveloperHand { .. }
        )
    }
}
