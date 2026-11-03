use super::evaluation::{HandEvaluator, rank_counts};
use crate::{
    DoudizhuAction, DoudizhuCard, DoudizhuPlay, DoudizhuPlayKind, DoudizhuRuleSet, hint_options,
};
use leocard_game_common::PlayerId;

pub(super) struct Candidate {
    pub(super) action: DoudizhuAction,
    pub(super) cost: f64,
    pub(super) bomb: bool,
    pub(super) rank: u8,
}

pub(super) struct PublicTurn<'a> {
    pub(super) actor: PlayerId,
    pub(super) landlord: PlayerId,
    pub(super) counts: &'a [usize],
    pub(super) table: Option<&'a (PlayerId, DoudizhuPlay)>,
    pub(super) rules: DoudizhuRuleSet,
}

impl PublicTurn<'_> {
    pub(super) fn candidates(
        &self,
        hand: &[DoudizhuCard],
        evaluator: &mut HandEvaluator,
    ) -> Vec<Candidate> {
        let original = rank_counts(hand);
        let teammate_leads = self.actor != self.landlord
            && self
                .table
                .is_some_and(|(leader, _)| *leader != self.landlord);
        let minimum_enemy = self
            .counts
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                (*index == usize::from(self.landlord)) != (self.actor == self.landlord)
            })
            .map(|(_, count)| *count)
            .min()
            .unwrap_or(33);
        let mut candidates = Vec::new();
        for play in hint_options(hand, self.table.map(|(_, play)| play), self.rules) {
            let bomb = matches!(play.kind, DoudizhuPlayKind::Bomb | DoudizhuPlayKind::Rocket);
            let remaining: Vec<_> = hand
                .iter()
                .copied()
                .filter(|card| !play.cards.contains(card))
                .collect();
            let mut cost = evaluator.cost(&remaining);
            if remaining.is_empty() {
                cost = -1000.0;
            } else {
                let counts = rank_counts(&remaining);
                let broken = (0..13)
                    .filter(|rank| original[*rank] >= 4 && counts[*rank] > 0 && counts[*rank] < 4)
                    .count();
                cost += broken as f64 * 1.4;
                cost += f64::from(play.rank) * 0.015;
                if bomb {
                    cost += 0.6 + play.cards.len() as f64 * 0.06;
                }
                if teammate_leads {
                    cost += if bomb { 2.5 } else { 1.1 };
                }
                if teammate_leads
                    && self
                        .table
                        .is_some_and(|(leader, _)| self.counts[usize::from(*leader)] <= 2)
                {
                    cost += 2.0;
                }
                // A lone enemy is best denied cheap single-card leads.
                if minimum_enemy == 1 && play.kind == DoudizhuPlayKind::Single {
                    cost += f64::from(14 - play.rank) * 0.16;
                }
                // The farmer immediately before the landlord is the last gatekeeper.
                if self.actor != self.landlord
                    && (self.actor.0 + 1) % 4 == self.landlord.0
                    && self.table.is_some()
                    && play.kind == DoudizhuPlayKind::Single
                {
                    cost -= f64::from(play.rank) * 0.035;
                }
                // Short teammates may be fed singles/pairs without knowing their ranks.
                if self.actor != self.landlord
                    && self.table.is_none()
                    && self.counts.iter().enumerate().any(|(index, count)| {
                        index != usize::from(self.actor)
                            && index != usize::from(self.landlord)
                            && *count == play.cards.len()
                            && *count <= 2
                    })
                {
                    cost -= 0.6;
                }
            }
            candidates.push(Candidate {
                action: DoudizhuAction::Play(play.cards),
                cost,
                bomb,
                rank: play.rank,
            });
        }
        if self.table.is_some() {
            candidates.push(Candidate {
                action: DoudizhuAction::Pass,
                cost: evaluator.cost(hand) + if teammate_leads { -0.35 } else { 0.4 },
                bomb: false,
                rank: 0,
            });
        }
        candidates.sort_by(|left, right| left.cost.total_cmp(&right.cost));
        candidates
    }
}

pub(super) fn shortlist(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    // Keep tactical alternatives even when decomposition scores prefer conserving them.
    let pass = candidates
        .iter()
        .position(|candidate| candidate.action == DoudizhuAction::Pass);
    let bomb = candidates.iter().position(|candidate| candidate.bomb);
    let strongest_bomb = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.bomb)
        .max_by_key(|(_, candidate)| match &candidate.action {
            DoudizhuAction::Play(cards) => (
                cards.iter().all(|card| card.rank() >= 13),
                cards.len(),
                candidate.rank,
            ),
            _ => unreachable!("bomb is a play"),
        })
        .map(|(index, _)| index);
    let block = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| !candidate.bomb)
        .max_by_key(|(_, candidate)| candidate.rank)
        .map(|(index, _)| index);
    let mut retained = Vec::new();
    for (index, candidate) in candidates.drain(..).enumerate() {
        if index < 8
            || Some(index) == pass
            || Some(index) == bomb
            || Some(index) == strongest_bomb
            || Some(index) == block
        {
            retained.push(candidate);
        }
    }
    retained
}
