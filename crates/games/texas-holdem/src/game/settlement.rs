use super::{
    TexasGameError, TexasGameState, TexasHandResult, TexasPhase, TexasPlayerState, TexasPotAward,
    TexasRunResult,
};
use crate::{TexasCard, evaluate_player_hand};
use leocard_game_common::PlayerId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

impl TexasGameState {
    pub(super) fn settle_uncontested(&mut self) -> TexasHandResult {
        self.refund_no_side_pot_excess();
        let winner = self.contenders()[0];
        let amount = self.pot();
        self.players[winner.index()].stack += amount;
        self.complete_hand(
            false,
            vec![TexasPotAward {
                amount,
                winners: vec![winner],
                winning_hand: None,
            }],
            None,
        )
    }

    pub(super) fn settle_showdown(
        &mut self,
        second_board: Option<Vec<TexasCard>>,
    ) -> Result<TexasHandResult, TexasGameError> {
        let pots = self.showdown_pots();
        let first_pots = pots
            .iter()
            .map(|(amount, eligible)| {
                (
                    if second_board.is_some() {
                        amount.div_ceil(2)
                    } else {
                        *amount
                    },
                    eligible.clone(),
                )
            })
            .collect::<Vec<_>>();
        let awards = self.award_board(&self.community.clone(), &first_pots)?;
        let second_run = if let Some(community) = second_board {
            let second_pots = pots
                .into_iter()
                .map(|(amount, eligible)| (amount / 2, eligible))
                .collect::<Vec<_>>();
            let awards = self.award_board(&community, &second_pots)?;
            Some(TexasRunResult { community, awards })
        } else {
            None
        };
        Ok(self.complete_hand(true, awards, second_run))
    }

    fn showdown_pots(&self) -> Vec<(u32, Vec<PlayerId>)> {
        let contenders = self.contenders();
        if !self.rules.side_pots {
            return vec![(self.pot(), contenders)];
        }
        let mut levels = self
            .players
            .iter()
            .map(TexasPlayerState::committed_total)
            .filter(|amount| *amount > 0)
            .collect::<Vec<_>>();
        levels.sort_unstable();
        levels.dedup();
        let mut previous = 0;
        levels
            .into_iter()
            .filter_map(|level| {
                let participants = self
                    .players
                    .iter()
                    .filter(|player| player.committed_total >= level)
                    .count() as u32;
                let amount = (level - previous) * participants;
                previous = level;
                let eligible = contenders
                    .iter()
                    .copied()
                    .filter(|player| self.players[player.index()].committed_total >= level)
                    .collect();
                (amount > 0).then_some((amount, eligible))
            })
            .collect()
    }

    fn award_board(
        &mut self,
        community: &[TexasCard],
        pots: &[(u32, Vec<PlayerId>)],
    ) -> Result<Vec<TexasPotAward>, TexasGameError> {
        let mut evaluated = vec![None; self.players.len()];
        for player in self.contenders() {
            evaluated[player.index()] = Some(evaluate_player_hand(
                &self.players[player.index()].hole_cards,
                community,
                &self.rules,
            )?);
        }
        let mut awards = Vec::new();
        for (amount, eligible) in pots {
            let best = eligible
                .iter()
                .filter_map(|player| evaluated[player.index()])
                .max_by(|left, right| left.cmp_with_rules(right, &self.rules))
                .expect("each pot has an eligible contender");
            let mut winners = eligible
                .iter()
                .copied()
                .filter(|player| {
                    evaluated[player.index()]
                        .is_some_and(|hand| hand.cmp_with_rules(&best, &self.rules).is_eq())
                })
                .collect::<Vec<_>>();
            winners.sort_by_key(|winner| self.clockwise_distance(self.dealer, *winner));
            let share = amount / winners.len() as u32;
            let remainder = amount % winners.len() as u32;
            for (index, winner) in winners.iter().enumerate() {
                self.players[winner.index()].stack += share + u32::from((index as u32) < remainder);
            }
            awards.push(TexasPotAward {
                amount: *amount,
                winners,
                winning_hand: Some(best),
            });
        }
        Ok(awards)
    }

    fn complete_hand(
        &mut self,
        showdown: bool,
        awards: Vec<TexasPotAward>,
        second_run: Option<TexasRunResult>,
    ) -> TexasHandResult {
        let mut result = TexasHandResult {
            dealer: self.dealer,
            showdown,
            community: self.community.clone(),
            awards,
            final_stacks: self.players.iter().map(TexasPlayerState::stack).collect(),
            second_run,
        };
        self.finish_squid(&mut result);
        self.current_player = None;
        self.phase = TexasPhase::Complete(result.clone());
        result
    }

    fn finish_squid(&mut self, result: &mut TexasHandResult) {
        let mut draw_seed = DefaultHasher::new();
        self.hand_number.hash(&mut draw_seed);
        self.deck.hash(&mut draw_seed);
        // Two-run policy: award one marker only for a sole winner scooping BOTH boards.
        // Split boards/ties earn no marker, independent of which board was dealt first.
        // Squid transfers and end-of-round settlement still happen exactly once.
        let squid_awards = if let Some(second) = &result.second_run {
            let first = &result.awards[0];
            let second = &second.awards[0];
            if first.winners.len() == 1 && first.winners == second.winners {
                vec![TexasPotAward {
                    amount: first.amount + second.amount,
                    winners: first.winners.clone(),
                    winning_hand: None,
                }]
            } else {
                Vec::new()
            }
        } else {
            result.awards.clone()
        };
        self.squid
            .finish_hand(&squid_awards, &mut result.final_stacks, draw_seed.finish());
        for (player, &stack) in self.players.iter_mut().zip(&result.final_stacks) {
            player.stack = stack;
        }
    }
}
