use super::appearance::deepseek_squid_marker;
use crate::{PotAward, TexasHoldemPlayerId, TexasHoldemRuleSet};
use std::cmp::Reverse;

/// A single newly awarded marker and the recipient's holdings in this round.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SquidAwardStatistics {
    pub deepseek: bool,
    /// Includes both ordinary and DeepSeek markers.
    pub owned: u8,
    pub deepseek_owned: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquidTransfer {
    pub from: TexasHoldemPlayerId,
    pub to: TexasHoldemPlayerId,
    pub amount: u32,
}

/// A round persists across hands. Its markers stay visible through settlement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SquidState {
    pub round: u32,
    pub markers: Vec<Option<TexasHoldemPlayerId>>,
    pub settled: bool,
    pub transfers: Vec<SquidTransfer>,
    last_award: Option<usize>,
}

impl SquidState {
    pub(crate) fn new(count: u8) -> Self {
        Self {
            round: 1,
            markers: vec![None; usize::from(count)],
            settled: false,
            transfers: Vec::new(),
            last_award: None,
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

    pub(crate) fn finish_hand(&mut self, awards: &[PotAward], stacks: &mut [u32]) {
        if self.markers.is_empty() || self.settled {
            return;
        }
        if let Some(award) = awards.first()
            && let [winner] = award.winners.as_slice()
            && award.amount > 0
            && let Some((index, marker)) = self
                .markers
                .iter_mut()
                .enumerate()
                .find(|(_, owner)| owner.is_none())
        {
            *marker = Some(*winner);
            self.last_award = Some(index);
        }
        if self.markers.iter().all(Option::is_some) || stacks.contains(&0) {
            self.settle(stacks);
        }
    }

    pub fn award_statistics(
        &self,
        seed: [u8; 16],
    ) -> Option<(TexasHoldemPlayerId, SquidAwardStatistics)> {
        let index = self.last_award?;
        let player = self.markers[index]?;
        let mut statistics = SquidAwardStatistics {
            deepseek: deepseek_squid_marker(seed, self.round, index).is_some(),
            owned: 0,
            deepseek_owned: 0,
        };
        for (index, owner) in self.markers.iter().enumerate() {
            if *owner == Some(player) {
                statistics.owned += 1;
                statistics.deepseek_owned +=
                    u8::from(deepseek_squid_marker(seed, self.round, index).is_some());
            }
        }
        Some((player, statistics))
    }

    fn settle(&mut self, stacks: &mut [u32]) {
        self.settled = true;
        let mut counts = vec![0_u32; stacks.len()];
        for owner in self.markers.iter().flatten() {
            counts[owner.0] += 1;
        }
        let total: u32 = counts.iter().sum();
        if total == 0 {
            return;
        }
        for loser in 0..stacks.len() {
            if counts[loser] != 0 {
                continue;
            }
            let payment = stacks[loser].min(total * TexasHoldemRuleSet::BIG_BLIND);
            stacks[loser] -= payment;
            let mut shares: Vec<_> = counts
                .iter()
                .map(|&count| payment * count / total)
                .collect();
            let mut remainder = payment - shares.iter().sum::<u32>();
            // Largest remainders preserve token proportions even when a payer is short.
            let mut recipients: Vec<_> = (0..counts.len())
                .filter(|&index| counts[index] > 0)
                .collect();
            recipients.sort_by_key(|&index| (Reverse(payment * counts[index] % total), index));
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
                self.transfers.push(SquidTransfer {
                    from: TexasHoldemPlayerId(loser),
                    to: TexasHoldemPlayerId(winner),
                    amount,
                });
            }
        }
    }
}
