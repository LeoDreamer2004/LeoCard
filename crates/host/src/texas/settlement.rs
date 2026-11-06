use super::{TexasSession, merge_texas_profile_stats};
use crate::features::{HandSettlement, MatchSettlement};
use leocard_texas_holdem::{TexasPhase, TexasProfileStats, reference_point_deltas};

impl MatchSettlement for TexasSession {
    type Outcome = ();

    fn commit_match(&mut self, _outcome: &Self::Outcome) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        let standings = game
            .players()
            .iter()
            .zip(game.game().players())
            .map(|(participant, state)| (participant.id, state.stack()))
            .collect::<Vec<_>>();
        let scores = standings
            .iter()
            .map(|(_, stack)| *stack)
            .collect::<Vec<_>>();
        let deltas = reference_point_deltas(&scores, game.rules().starting_chips)
            .expect("a Texas Hold'em table always contains between three and eight players");
        let settlements = standings
            .iter()
            .zip(deltas)
            .map(|((player, _), delta)| (*player, delta))
            .collect::<Vec<_>>();
        let match_profile_stats = self.match_profile_stats.clone();
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |participant, delta| {
                let player_id = participant.id;
                let final_chips = standings
                    .iter()
                    .find_map(|(player, chips)| (*player == player_id).then_some(*chips))
                    .expect("a settlement player has final chips");
                let current_profile_stats = match_profile_stats.get(usize::from(player_id.0));
                let placement = 1 + scores.iter().filter(|other| **other > final_chips).count();
                let aggregate = participant
                    .game_profiles
                    .texas
                    .get_or_insert_with(TexasProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                aggregate.total_final_chips = aggregate
                    .total_final_chips
                    .saturating_add(u64::from(final_chips));
                if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = current_profile_stats {
                    merge_texas_profile_stats(aggregate, current);
                }
            },
        );
    }
}

impl HandSettlement for TexasSession {
    type Outcome = bool;
    type Output = ();

    fn commit_hand(&mut self, was_complete: &Self::Outcome) {
        let was_complete = *was_complete;

        if was_complete
            || !self
                .game
                .as_ref()
                .is_some_and(|game| matches!(game.game().phase(), TexasPhase::Complete(_)))
        {
            return;
        }
        self.record_completed_hand();
        self.room.prepare_rematch();
        self.automatic_action_delay = None;
        if self.tournament_complete() {
            self.commit_match(&());
        }
    }
}
