use super::MinecraftGuessSession;
use leocard_minecraft_guess::reference_point_deltas;
use leocard_protocol::PlayerId;

impl MinecraftGuessSession {
    pub(super) fn settle_match(&mut self) {
        let state = self.match_state.as_ref().expect("active match");
        if !state.progress().is_last_game()
            || !self.game.as_ref().is_some_and(|game| game.finished())
        {
            return;
        }
        let scores: Vec<_> = state.scores().collect();
        let values: Vec<_> = scores.iter().map(|(_, score)| *score).collect();
        let deltas =
            reference_point_deltas(&values, self.rules.rounds).expect("two to six players");
        let settlements = scores
            .into_iter()
            .zip(deltas)
            .map(|((player, _), delta)| (PlayerId(player), delta));
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |_, _| {},
        );
    }
}
