use super::{QiGui523Session, analysis::profile_statistics, merge_qigui523_play_stats};
use leocard_protocol::QiGui523ProfileStats;
use leocard_qigui523::{Phase, reference_point_deltas};

impl QiGui523Session {
    pub(super) fn apply_finished_reference_points(&mut self) {
        let Some(scores) = self.game.as_ref().and_then(|game| match game.phase() {
            Phase::Finished(result) => Some(result.scores.clone()),
            Phase::Playing => None,
        }) else {
            return;
        };
        let deltas = reference_point_deltas(&scores)
            .expect("a running game always contains between two and six players");
        let settlements = self
            .players
            .iter()
            .zip(deltas)
            .map(|(player, delta)| (player.id, delta))
            .collect::<Vec<_>>();
        let match_profile_stats = self
            .statistics
            .as_ref()
            .map(|statistics| {
                statistics
                    .players()
                    .iter()
                    .map(profile_statistics)
                    .collect::<Vec<_>>()
            })
            .expect("an active game has statistics");
        let applied = self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |player, delta| {
                let index = usize::from(player.id.0);
                let score = scores[index];
                let placement = 1 + scores.iter().filter(|other| **other > score).count();
                let aggregate = player
                    .game_profiles
                    .qigui523
                    .get_or_insert_with(QiGui523ProfileStats::default);
                aggregate.completed_games = aggregate.completed_games.saturating_add(1);
                aggregate.total_score = aggregate.total_score.saturating_add(u64::from(score));
                aggregate.total_reference_delta = aggregate
                    .total_reference_delta
                    .saturating_add(i64::from(delta));
                if let Some(count) = aggregate.placement_counts.get_mut(placement - 1) {
                    *count = count.saturating_add(1);
                }
                if let Some(current) = match_profile_stats.get(index) {
                    merge_qigui523_play_stats(aggregate, current);
                }
            },
        );
        if applied {
            self.room.prepare_rematch();
        }
    }
}
