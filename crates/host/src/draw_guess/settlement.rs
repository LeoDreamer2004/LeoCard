use super::DrawGuessSession;
use crate::features::MatchSettlement;
use leocard_draw_guess::DrawGuessPhase;
use leocard_game_common::guessing_reference_point_deltas;

impl MatchSettlement for DrawGuessSession {
    type Outcome = ();

    fn commit_match(&mut self, _outcome: &Self::Outcome) {
        let game = self.game.as_ref().expect("active drawing game");
        if game.phase() != DrawGuessPhase::Finished {
            return;
        }
        let scores: Vec<_> = game.players().iter().map(|player| player.score).collect();
        let deltas = guessing_reference_point_deltas(&scores, self.rules.rounds)
            .expect("two to eight players and validated round count");
        let settlements = game.players().iter().map(|player| player.id).zip(deltas);
        self.room.settle_completed_match_profiles_once(
            &mut self.finished_reference_changes,
            settlements,
            |_, _| {},
        );
    }
}
