use crate::{EvaluatedHand, GameState, HandResult, TexasHoldemPlayerId};
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct PotStatistics {
    pub won_chips: u32,
    pub won_pot: u32,
    pub won_side_pot_chips: u32,
    pub defeated: HashSet<usize>,
}

impl PotStatistics {
    pub(super) fn new(
        game: &GameState,
        result: &HandResult,
        player: TexasHoldemPlayerId,
        evaluated: &[Option<EvaluatedHand>],
    ) -> Self {
        let mut pots = Self::default();
        if result.showdown {
            let mut levels = game
                .players()
                .iter()
                .map(|other| other.committed_total())
                .filter(|level| *level > 0)
                .collect::<Vec<_>>();
            levels.sort_unstable();
            levels.dedup();
            for (pot_index, (level, award)) in levels.into_iter().zip(&result.awards).enumerate() {
                // A one-contributor pot is an uncalled wager returned to its owner.
                if game
                    .players()
                    .iter()
                    .filter(|other| other.committed_total() >= level)
                    .count()
                    < 2
                {
                    continue;
                }
                let Some(winner_index) = award.winners.iter().position(|winner| *winner == player)
                else {
                    continue;
                };
                let received = award.amount / award.winners.len() as u32
                    + u32::from((winner_index as u32) < award.amount % award.winners.len() as u32);
                if received == 0 {
                    continue;
                }
                pots.won_chips += received;
                pots.won_pot += award.amount;
                if pot_index > 0 {
                    pots.won_side_pot_chips += received;
                }
                for other in game.players().iter().filter(|other| {
                    other.id() != player && !other.folded() && other.committed_total() >= level
                }) {
                    if let (Some(own), Some(opponent)) =
                        (evaluated[player.0], evaluated[other.id().0])
                        && own.cmp_with_rules(&opponent, game.rules()).is_gt()
                    {
                        pots.defeated.insert(other.id().0);
                    }
                }
            }
        } else {
            let invested = game
                .players()
                .iter()
                .map(|other| other.committed_total())
                .collect::<Vec<_>>();
            let mut ordered = invested.clone();
            ordered.sort_unstable_by(|left, right| right.cmp(left));
            let uncalled = invested[player.0].saturating_sub(ordered[1]);
            if result
                .awards
                .iter()
                .any(|award| award.winners.contains(&player))
            {
                pots.won_chips =
                    result.awards.iter().map(|award| award.amount).sum::<u32>() - uncalled;
                pots.won_pot = pots.won_chips;
            }
        }
        pots
    }
}
