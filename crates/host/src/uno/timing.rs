use super::UnoSession;
use crate::action_timing::{
    ActionTimingRequest, PendingAction, attach_action_timing, sync_action_timing,
};
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command};
use crate::{ConnectionId, Delivery};
use leocard_protocol::{
    ClientCommand, ClientMessage, GameCommand, GameSnapshot, UnoCommand, UnoPendingSwapView,
    UnoPhaseView,
};

impl UnoSession {
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
                .uno
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
            GameSnapshot::Uno(s) if matches!(s.phase, UnoPhaseView::Playing) => {
                // None is used during automatic draw reveals; do not time the animation.
                let Some(current) = s.current_player else {
                    return Vec::new();
                };
                let player = match s.pending_swap {
                    Some(
                        UnoPendingSwapView::SwapOneTarget { player }
                        | UnoPendingSwapView::SwapOneGive { player, .. }
                        | UnoPendingSwapView::ForceTrade { player }
                        | UnoPendingSwapView::ChooseColor { player }
                        | UnoPendingSwapView::SevenSwap { player }
                        | UnoPendingSwapView::ColorRoulette { player },
                    ) => player,
                    None => current,
                };
                let actor = s
                    .players
                    .iter()
                    .find(|p| p.player.id == player && !p.eliminated);
                // Public hand length distinguishes drawing then choosing without depending on `you`.
                // Jump-in, UNO calls and reports are optional, not additional required actors.
                PendingAction::for_players(
                    actor.map(|p| p.player.id),
                    (
                        "uno",
                        s.match_id,
                        s.pending_swap,
                        (s.discard_top, s.current_color, s.flip_side, s.direction),
                        (
                            s.pending_draw,
                            s.pending_kind,
                            s.challenge_offender,
                            s.pending_skip,
                            s.sudden_death_free_play,
                        ),
                        actor.map(|p| p.hand_len),
                    ),
                )
            }
            _ => Vec::new(),
        }
    }

    fn is_decision(command: &GameCommand) -> bool {
        let GameCommand::Uno(command) = command else {
            return false;
        };
        !matches!(
            command,
            UnoCommand::SetAutoPlay { .. }
                | UnoCommand::UpdateRules { .. }
                | UnoCommand::JumpIn { .. }
                | UnoCommand::CallUno
                | UnoCommand::ReportUno { .. }
        )
    }
}
