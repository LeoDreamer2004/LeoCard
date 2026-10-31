use super::appearance::deepseek_squid_marker;
use crate::{TexasPotAward, TexasRuleSet};
use fastrand::Rng;
use leocard_game_common::PlayerId;

use std::cmp::Reverse;

/// A single newly awarded marker and the recipient's holdings in this round.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TexasSquidAwardStatistics {
    pub deepseek: bool,
    /// Includes both ordinary and DeepSeek markers.
    pub owned: u8,
    pub deepseek_owned: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TexasSquidTransfer {
    pub from: PlayerId,
    pub to: PlayerId,
    pub amount: u32,
}

/// A round persists across hands. Its markers stay visible through settlement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TexasSquidState {
    pub round: u32,
    pub markers: Vec<Option<PlayerId>>,
    pub settled: bool,
    pub transfers: Vec<TexasSquidTransfer>,
    last_award: Option<usize>,
    seed: [u8; 16],
    deepseek_double: bool,
}

impl TexasSquidState {
    pub(crate) fn new(count: u8, seed: [u8; 16], deepseek_double: bool) -> Self {
        Self {
            round: 1,
            markers: vec![None; usize::from(count)],
            settled: false,
            transfers: Vec::new(),
            last_award: None,
            seed,
            deepseek_double,
        }
    }

    pub(crate) fn start_hand(&mut self) {
        self.transfers.clear();
        self.last_award = None;
        if self.settled {
            self.round += 1;
            self.markers.fill(None);
            self.settled = false;
        }
    }

    pub(crate) fn finish_hand(
        &mut self,
        awards: &[TexasPotAward],
        stacks: &mut [u32],
        draw_seed: u64,
    ) {
        if self.markers.is_empty() || self.settled {
            return;
        }
        if let Some(award) = awards.first()
            && let [winner] = award.winners.as_slice()
            && award.amount > 0
        {
            let available = self
                .markers
                .iter()
                .enumerate()
                .filter_map(|(index, owner)| owner.is_none().then_some(index))
                .collect::<Vec<_>>();
            if let Some(index) = Rng::with_seed(draw_seed).choice(available) {
                self.markers[index] = Some(*winner);
                self.last_award = Some(index);
            }
        }
        if self.markers.iter().all(Option::is_some) || stacks.contains(&0) {
            self.settle(stacks);
        }
    }

    pub fn award_statistics(&self) -> Option<(PlayerId, TexasSquidAwardStatistics)> {
        let index = self.last_award?;
        let player = self.markers[index]?;
        let mut statistics = TexasSquidAwardStatistics {
            deepseek: deepseek_squid_marker(self.seed, self.round, index).is_some(),
            owned: 0,
            deepseek_owned: 0,
        };
        for (index, owner) in self.markers.iter().enumerate() {
            if *owner == Some(player) {
                statistics.owned += 1;
                statistics.deepseek_owned +=
                    u8::from(deepseek_squid_marker(self.seed, self.round, index).is_some());
            }
        }
        Some((player, statistics))
    }

    fn settle(&mut self, stacks: &mut [u32]) {
        self.settled = true;
        let mut weights = vec![0_u32; stacks.len()];
        for (index, owner) in self.markers.iter().enumerate() {
            if let Some(owner) = owner {
                weights[owner.index()] += 1 + u32::from(
                    self.deepseek_double
                        && deepseek_squid_marker(self.seed, self.round, index).is_some(),
                );
            }
        }
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return;
        }
        for loser in 0..stacks.len() {
            if weights[loser] != 0 {
                continue;
            }
            let payment = stacks[loser].min(total * TexasRuleSet::BIG_BLIND);
            stacks[loser] -= payment;
            let mut shares: Vec<_> = weights
                .iter()
                .map(|&weight| payment * weight / total)
                .collect();
            let mut remainder = payment - shares.iter().sum::<u32>();
            // Largest remainders preserve token proportions even when a payer is short.
            let mut recipients: Vec<_> = (0..weights.len())
                .filter(|&index| weights[index] > 0)
                .collect();
            recipients.sort_by_key(|&index| (Reverse(payment * weights[index] % total), index));
            for &index in &recipients {
                if remainder == 0 {
                    break;
                }
                shares[index] += 1;
                remainder -= 1;
            }
            for (winner, amount) in shares
                .into_iter()
                .enumerate()
                .filter(|(_, amount)| *amount > 0)
            {
                stacks[winner] += amount;
                self.transfers.push(TexasSquidTransfer {
                    from: PlayerId::from_index(loser),
                    to: PlayerId::from_index(winner),
                    amount,
                });
            }
        }
    }
}
