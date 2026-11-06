use super::GuandanSession;
use crate::features::MatchSettlement;
use leocard_game_common::PlayerId;
use leocard_guandan::GuandanProfileStats;

impl MatchSettlement for GuandanSession {
    type Outcome = ();

    fn commit_match(&mut self, _outcome: &Self::Outcome) {
        let winner = self
            .match_state
            .as_ref()
            .expect("active match")
            .previous_result()
            .expect("completed match")
            .winning_team;
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            (0..4).map(|index| (PlayerId(index), 0)),
            |player, delta| {
                let aggregate = player
                    .game_profiles
                    .guandan
                    .get_or_insert_with(GuandanProfileStats::default);
                self.statistics
                    .merge_into(player.id, aggregate, player.id.0 % 2 == winner, delta);
            },
        );
    }
}
