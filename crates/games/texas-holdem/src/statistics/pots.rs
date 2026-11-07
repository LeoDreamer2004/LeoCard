use crate::{
    TexasEvaluatedHand, TexasGameState, TexasHandResult, TexasPotAward, TexasRank,
    evaluate_player_hand,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[derive(Default)]
pub(super) struct PotStatistics {
    pub won_chips: u32,
    pub won_pot: u32,
    pub won_side_pot_chips: u32,
    pub tied_winners: u8,
    pub defeated: HashSet<usize>,
    pub lost_to_lower_unpaired_hand: bool,
    pub won_each_board: bool,
    pub lost_first_won_second: bool,
}

impl PotStatistics {
    fn single_pot(game: &TexasGameState, result: &TexasHandResult, player: PlayerId) -> Self {
        let mut pots = Self::default();
        let won_board = |awards: &[TexasPotAward]| {
            awards
                .iter()
                .any(|award| award.amount > 0 && award.winners == [player])
        };
        pots.won_each_board = won_board(&result.awards);
        if let Some(second) = &result.second_run {
            let won_second = won_board(&second.awards);
            pots.won_each_board &= won_second;
            pots.lost_first_won_second = won_second
                && !result
                    .awards
                    .iter()
                    .any(|award| award.winners.contains(&player));
        }
        let mut received = vec![0; game.players().len()];
        let mut not_defeated_on_both = HashSet::new();
        let runs = [(result.community.as_slice(), result.awards.as_slice())]
            .into_iter()
            .chain(
                result
                    .second_run
                    .iter()
                    .map(|run| (run.community.as_slice(), run.awards.as_slice())),
            );
        for (community, awards) in runs {
            let evaluated = game
                .players()
                .iter()
                .map(|other| evaluate_player_hand(other.hole_cards(), community, game.rules()).ok())
                .collect::<Vec<_>>();
            for award in awards {
                for (index, winner) in award.winners.iter().enumerate() {
                    let share = award.amount / award.winners.len() as u32
                        + u32::from((index as u32) < award.amount % award.winners.len() as u32);
                    received[winner.index()] += share;
                    if *winner == player && share > 0 {
                        pots.won_pot += award.amount;
                        if award.winners.len() > 1 {
                            pots.tied_winners = pots.tied_winners.max(award.winners.len() as u8);
                        }
                    }
                }
                if game.players()[player.index()].folded() {
                    continue;
                }
                pots.lost_to_lower_unpaired_hand |=
                    Self::lost_to_unpaired_winner(game, award, player, &evaluated);
                for other in game
                    .players()
                    .iter()
                    .filter(|other| !other.folded() && other.id() != player)
                {
                    let Some((own, opponent)) =
                        evaluated[player.index()].zip(evaluated[other.id().index()])
                    else {
                        continue;
                    };
                    if !own.cmp_with_rules(&opponent, game.rules()).is_gt() {
                        not_defeated_on_both.insert(other.id().index());
                    }
                    if award.winners.contains(&player)
                        && own.cmp_with_rules(&opponent, game.rules()).is_gt()
                    {
                        pots.defeated.insert(other.id().index());
                    }
                }
            }
        }
        pots.won_chips = received[player.index()];
        if result.second_run.is_some() {
            // Multi-run wins/loss achievements use the net aggregate poker result, not one board.
            // Refunds are already excluded from committed_total; squid transfers are separate.
            let invested = game.players()[player.index()].committed_total();
            pots.lost_to_lower_unpaired_hand &= received[player.index()] < invested;
            pots.won_chips = pots.won_chips.saturating_sub(invested);
            pots.defeated.retain(|&index| {
                // Defeated-category achievements require beating this opponent on both boards.
                pots.won_chips > 0
                    && received[index] < game.players()[index].committed_total()
                    && !not_defeated_on_both.contains(&index)
            });
        }
        pots
    }

    fn lost_to_unpaired_winner(
        game: &TexasGameState,
        award: &TexasPotAward,
        player: PlayerId,
        evaluated: &[Option<TexasEvaluatedHand>],
    ) -> bool {
        award.winners.iter().any(|winner| {
            let other = &game.players()[winner.index()];
            let [first, second] = other.hole_cards() else {
                return false;
            };
            first.rank() != second.rank()
                && first.rank() < TexasRank::Ace
                && second.rank() < TexasRank::Ace
                && evaluated[player.index()]
                    .zip(evaluated[winner.index()])
                    .is_some_and(|(own, opponent)| {
                        own.cmp_with_rules(&opponent, game.rules()).is_lt()
                    })
        })
    }

    pub(super) fn new(
        game: &TexasGameState,
        result: &TexasHandResult,
        player: PlayerId,
        evaluated: &[Option<TexasEvaluatedHand>],
    ) -> Self {
        if result.showdown && !game.rules().side_pots {
            return Self::single_pot(game, result, player);
        }
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
                let own = &game.players()[player.index()];
                if !own.folded() && own.committed_total() >= level {
                    pots.lost_to_lower_unpaired_hand |=
                        Self::lost_to_unpaired_winner(game, award, player, evaluated);
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
                if award.winners.len() > 1 {
                    pots.tied_winners = pots.tied_winners.max(award.winners.len() as u8);
                }
                pots.won_pot += award.amount;
                if pot_index > 0 {
                    pots.won_side_pot_chips += received;
                }
                for other in game.players().iter().filter(|other| {
                    other.id() != player && !other.folded() && other.committed_total() >= level
                }) {
                    if let (Some(own), Some(opponent)) =
                        (evaluated[player.index()], evaluated[other.id().index()])
                        && own.cmp_with_rules(&opponent, game.rules()).is_gt()
                    {
                        pots.defeated.insert(other.id().index());
                        pots.won_each_board |= award.winners.len() == 1;
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
            let uncalled = invested[player.index()].saturating_sub(ordered[1]);
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
