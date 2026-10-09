use super::{
    UnoSession, analysis::profile_statistics, analysis_events, append_finished_event,
    events_for_outcome, merge_uno_profile_stats, pending_draw_reveal_for_outcome, to_core_player,
};
use crate::{ConnectionId, Delivery, lifecycle::HostedGameLifecycle};
use leocard_protocol::{
    GameViolation, PlayerId, PlayerViolation, RejectReason, RequestId, UnoProfileStats,
};
use leocard_uno::{
    ActionOutcome, GameError, GameState, Phase, UnoActionContext, UnoCard, UnoColor, UnoPlayerId,
};

impl UnoSession {
    pub(super) fn perform_action<F>(
        &mut self,
        connection: ConnectionId,
        request_id: RequestId,
        played: Vec<(UnoCard, Option<UnoColor>)>,
        successful_jump_in: bool,
        action: F,
    ) -> Vec<Delivery>
    where
        F: FnOnce(&mut GameState, UnoPlayerId) -> Result<ActionOutcome, GameError>,
    {
        let Some(player) = self.room.player_id(connection) else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Player(PlayerViolation::NotJoined),
            );
        };
        let Some(game) = self.game.as_mut() else {
            return self.room.reject(
                connection,
                request_id,
                RejectReason::Game(GameViolation::GameNotStarted),
            );
        };
        let jump_in_was_available =
            successful_jump_in && game.jump_in_card(to_core_player(player)).is_some();
        let before = UnoActionContext::capture(game, to_core_player(player));
        match action(game, to_core_player(player)) {
            Ok(outcome) => {
                let mut events = events_for_outcome(&outcome, &played);
                append_finished_event(&mut events, game);
                let before = before.expect("accepted UNO actions started in an active game");
                events.extend(analysis_events(
                    self.statistics
                        .as_mut()
                        .expect("active UNO games have a recorder")
                        .observe(before, &outcome, &played, jump_in_was_available, game),
                ));
                self.apply_finished_reference_points();
                self.reset_auto_play_delay();
                self.room.bump_revision();
                let draw_reveal = pending_draw_reveal_for_outcome(
                    &outcome,
                    self.game
                        .as_ref()
                        .expect("a successful UNO action keeps an active game"),
                );
                let mut deliveries = self.broadcast_events(std::mem::take(&mut events));
                if draw_reveal.is_some() || self.pending_draw_reveal.is_none() {
                    self.pending_draw_reveal = draw_reveal;
                }
                deliveries.extend(self.broadcast_game(Some((connection, request_id))));
                deliveries
            }
            Err(error) => self.reject_game_error(connection, request_id, &error),
        }
    }

    pub(super) fn apply_finished_reference_points(&mut self) {
        let result = match self.game.as_ref().map(GameState::phase) {
            Some(Phase::Finished(result)) => result.clone(),
            _ => return,
        };
        let eliminated = self
            .game
            .as_ref()
            .expect("a finished UNO result belongs to a game")
            .players()
            .iter()
            .map(|player| player.eliminated())
            .collect::<Vec<_>>();
        let statistics = self
            .statistics
            .as_ref()
            .expect("a finished UNO game has a recorder");
        let match_profile_stats = (0..result.hand_scores.len())
            .map(|index| {
                profile_statistics(
                    statistics
                        .player_statistics(UnoPlayerId(index))
                        .expect("recorder and game players stay aligned"),
                )
            })
            .collect::<Vec<_>>();
        let settlements = result
            .reference_deltas
            .iter()
            .copied()
            .enumerate()
            .map(|(index, delta)| (PlayerId(index as u8), delta))
            .collect::<Vec<_>>();
        let applied = self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |participant, delta| {
                let index = usize::from(participant.id.0);
                let aggregate = participant
                    .game_profiles
                    .uno
                    .get_or_insert_with(UnoProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                if !eliminated[index]
                    && let Some(score) = result.hand_scores.get(index)
                {
                    aggregate.total_remaining_score = aggregate
                        .total_remaining_score
                        .saturating_add(u64::from(*score));
                }
                if let Some(placement) = result.placements.get(index).copied()
                    && let Some(count) = aggregate
                        .placement_counts
                        .get_mut(usize::from(placement.saturating_sub(1)))
                {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = match_profile_stats.get(index) {
                    merge_uno_profile_stats(aggregate, current);
                }
            },
        );
        if applied {
            self.room.prepare_rematch();
        }
    }
}
