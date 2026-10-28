use super::state::{DoudizhuAchievementTracker, is_bomb};
use crate::achievements::{DoudizhuAchievementFacts, DoudizhuHandFacts, DoudizhuMatchFacts};
use crate::{GameState, Phase};

impl DoudizhuAchievementTracker {
    pub fn finish_hand(
        &mut self,
        game: &GameState,
        scores_before: &[i32],
        scores_after: &[i32],
        match_complete: bool,
    ) -> Vec<(u8, DoudizhuAchievementFacts)> {
        assert_eq!(game.phase, Phase::Finished);
        let result = game.result.as_ref().expect("finished hand has a result");
        let landlord = game.landlord.expect("finished hand has a landlord");
        let mut events = Vec::new();
        for (index, player) in self.players.iter_mut().enumerate() {
            let is_landlord = index == usize::from(landlord);
            let won = is_landlord == result.landlord_won;
            player.perfect_spring &= won && result.spring;
            events.push((
                index as u8,
                DoudizhuAchievementFacts::Hand(DoudizhuHandFacts {
                    landlord: is_landlord,
                    won,
                    spring: result.spring,
                    finished_cards: result.winner == index as u8,
                    finishing_bomb: result.winner == index as u8
                        && game
                            .table
                            .as_ref()
                            .is_some_and(|(_, play)| is_bomb(play.kind)),
                    unavailable_exchange: player.hand.unavailable_exchange,
                    all_bomb_hand: player.hand.all_bomb_hand,
                    play_count: player.hand.play_count,
                    score: result.scores[index],
                }),
            ));
            if match_complete {
                events.push((
                    index as u8,
                    DoudizhuAchievementFacts::Match(DoudizhuMatchFacts {
                        score: scores_after[index],
                        perfect_spring: player.perfect_spring,
                        first_to_last: scores_before
                            .iter()
                            .all(|score| *score <= scores_before[index])
                            && scores_before
                                .iter()
                                .any(|score| *score < scores_before[index])
                            && scores_after
                                .iter()
                                .all(|score| *score >= scores_after[index])
                            && scores_after
                                .iter()
                                .any(|score| *score > scores_after[index]),
                    }),
                ));
            }
        }
        events
    }
}
