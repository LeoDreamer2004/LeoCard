use super::DoudizhuSession;
use crate::features::{HandSettlement, MatchSettlement};
use leocard_doudizhu::DoudizhuProfileStats;
use leocard_game_common::PlayerId;
use leocard_protocol::DoudizhuEvent;

impl MatchSettlement for DoudizhuSession {
    type Outcome = ();

    fn commit_match(&mut self, _outcome: &Self::Outcome) {
        if let Some(deltas) = self
            .match_state
            .as_ref()
            .expect("active match")
            .reference_deltas(self.rules.mode)
        {
            self.room.settle_completed_match_profiles_once(
                &mut self.finished_reference_changes,
                deltas
                    .into_iter()
                    .enumerate()
                    .map(|(player, delta)| (PlayerId(player as u8), delta)),
                |player, delta| {
                    let stats = player
                        .game_profiles
                        .doudizhu
                        .get_or_insert_with(DoudizhuProfileStats::default);
                    self.statistics.merge_into(player.id, stats, delta);
                },
            );
        }
    }
}

impl HandSettlement for DoudizhuSession {
    type Outcome = ();
    type Output = Vec<DoudizhuEvent>;

    fn commit_hand(&mut self, _outcome: &Self::Outcome) -> Self::Output {
        let game = self.game.as_ref().expect("active game");
        let result = game.result.as_ref().expect("finished hand");
        let state = self.match_state.as_mut().expect("active match");
        let before = state.scores().to_vec();
        assert!(state.finish_hand(result));
        let achievements =
            self.achievements
                .finish_hand(game, &before, state.scores(), state.complete());
        self.statistics.finish_hand(
            game.landlord.expect("finished hand has a landlord"),
            &result.scores,
        );
        MatchSettlement::commit_match(self, &());
        self.room.prepare_rematch();
        achievements
            .into_iter()
            .map(|(player, statistics)| DoudizhuEvent::ActionAnalyzed {
                player,
                statistics: Box::new(statistics),
            })
            .collect()
    }
}
