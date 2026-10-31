use super::candidates::{plays, unique};
use super::hand::{category_index, hand_value};
use super::knowledge::{Knowledge, PublicKnowledge};
use crate::play::classify_cards;
use crate::{
    ShengjiCard, ShengjiCategory, ShengjiClassifiedPlay, ShengjiRuleSet, ShengjiTeamId,
    ShengjiTrickRecord, ShengjiTrump, compare_for_trick,
};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct PositionView<'a> {
    pub player: PlayerId,
    pub hand: &'a [ShengjiCard],
    pub trump: ShengjiTrump,
    pub rules: ShengjiRuleSet,
    pub trick: Option<&'a ShengjiTrickRecord>,
    pub public: &'a PublicKnowledge,
    pub lengths: &'a [usize],
    pub collecting_score: u32,
    pub bottom: Option<&'a [ShengjiCard]>,
}

pub(super) struct Evaluation<'a> {
    view: PositionView<'a>,
    knowledge: Knowledge<'a>,
    value: f64,
    remaining: Vec<PlayerId>,
    side_controls: usize,
}

impl<'a> Evaluation<'a> {
    pub fn new(view: PositionView<'a>) -> Self {
        let remaining = (1..view.rules.player_count())
            .map(|offset| PlayerId((view.player.0 + offset) % view.rules.player_count()))
            .take(
                view.trick
                    .map_or(usize::from(view.rules.player_count()) - 1, |trick| {
                        usize::from(view.rules.player_count()) - trick.plays.len() - 1
                    }),
            )
            .collect();
        let knowledge = Knowledge::new(
            view.player,
            view.hand,
            view.trump,
            view.public,
            view.lengths,
            view.bottom,
        );
        let side_controls = view
            .hand
            .iter()
            .filter(|card| !view.trump.is_trump(**card))
            .filter(|card| {
                knowledge.master(&classify_cards(&[**card], view.trump).expect("single"))
            })
            .count();
        Self {
            knowledge,
            value: hand_value(view.hand, view.trump),
            view,
            remaining,
            side_controls,
        }
    }

    pub fn ranked(&self, limit: usize) -> Vec<(ShengjiClassifiedPlay, f64)> {
        let lead = self
            .view
            .trick
            .and_then(|trick| trick.plays.first())
            .map(|(_, play)| play);
        let mut choices = plays(
            self.view.hand,
            lead,
            self.view.trump,
            if limit >= 12 { 96 } else { 16 },
        );
        if lead.is_none() && self.view.rules.allow_throw {
            self.add_throws(&mut choices);
        }
        unique(&mut choices);
        let mut ranked: Vec<_> = choices
            .into_iter()
            .map(|play| {
                let value = self.score(&play);
                (play, value)
            })
            .collect();
        ranked.sort_by(|(_, left), (_, right)| right.total_cmp(left));
        if ranked.len() > limit {
            // Preserve distinct plans instead of filling the shortlist with
            // nearly identical cards from one suit or one shape.
            let mut selected = Vec::new();
            let mut families = HashSet::new();
            for (index, (play, _)) in ranked.iter().enumerate() {
                let fresh = families.insert((play.category, play.cards.len(), play.is_throw()));
                if index < limit.div_ceil(2) || fresh {
                    selected.push(index);
                }
                if selected.len() == limit {
                    break;
                }
            }
            for index in 0..ranked.len() {
                if selected.len() == limit {
                    break;
                }
                if !selected.contains(&index) {
                    selected.push(index);
                }
            }
            ranked = ranked
                .into_iter()
                .enumerate()
                .filter_map(|(index, value)| selected.contains(&index).then_some(value))
                .collect();
        }
        ranked
    }

    fn add_throws(&self, choices: &mut Vec<ShengjiClassifiedPlay>) {
        for cat in 0..5 {
            let cards: Vec<_> = self
                .view
                .hand
                .iter()
                .copied()
                .filter(|card| category_index(*card, self.view.trump) == cat)
                .collect();
            if cards.is_empty() {
                continue;
            }
            if let Some(play) = self.knowledge.safe_throw(&cards, self.view.rules) {
                choices.push(play);
            }
            let winners: Vec<_> = choices
                .iter()
                .filter(|play| {
                    !play.is_throw()
                        && category_index(play.cards[0], self.view.trump) == cat
                        && self.knowledge.master(play)
                })
                .cloned()
                .collect();
            let mut added = 0;
            for (index, left) in winners.iter().enumerate() {
                for right in winners.iter().skip(index + 1) {
                    if added == 24 {
                        break;
                    }
                    if left.cards.iter().any(|card| right.cards.contains(card)) {
                        continue;
                    }
                    let mut cards = left.cards.clone();
                    cards.extend_from_slice(&right.cards);
                    if let Some(play) = self.knowledge.safe_throw(&cards, self.view.rules) {
                        choices.push(play);
                        added += 1;
                    }
                }
            }
        }
    }

    pub fn score(&self, play: &ShengjiClassifiedPlay) -> f64 {
        let remaining: Vec<_> = self
            .view
            .hand
            .iter()
            .copied()
            .filter(|card| !play.cards.contains(card))
            .collect();
        let cost = self.value - hand_value(&remaining, self.view.trump);
        self.view.trick.map_or_else(
            || self.lead_score(play, cost),
            |trick| self.follow_score(play, trick, cost),
        )
    }

    fn lead_score(&self, play: &ShengjiClassifiedPlay, cost: f64) -> f64 {
        let trumping = play.category == ShengjiCategory::Trump;
        let master = self.knowledge.master(play);
        let confidence =
            self.knowledge
                .team_probability(play, play, self.view.player, &self.remaining);
        let points = points(play);
        let mut value =
            20.0 * confidence + points * (confidence * 2.0 - 1.0) + play.cards.len() as f64 * 1.7
                - cost * 0.8;
        let length = self
            .view
            .hand
            .iter()
            .filter(|card| {
                category_index(**card, self.view.trump)
                    == category_index(play.cards[0], self.view.trump)
            })
            .count();
        if !trumping {
            if master {
                value += 16.0 * confidence + 5.0 / (1 + length - play.cards.len()) as f64;
            }
            if play.cards.len() >= 2 {
                value += 3.0 + play.cards.len() as f64;
            }
            if length >= 5 && master {
                value += (length - 4) as f64 * 1.2;
            }
            for &seat in &self.remaining {
                if ShengjiTeamId::of(seat) == ShengjiTeamId::of(self.view.player)
                    && self.view.public.voids[usize::from(seat.0)]
                        [category_index(play.cards[0], self.view.trump)]
                    && play.cards.len() == 1
                {
                    value += 5.0 * confidence;
                }
            }
        } else {
            let trumps = self
                .view
                .hand
                .iter()
                .filter(|card| self.view.trump.is_trump(**card))
                .count();
            let average = self.knowledge.outside_count(4) as f64
                / f64::from(self.view.rules.player_count() - 1);
            let purposeful =
                trumps as f64 > average * 1.35 && trumps >= 4 && self.side_controls > 0;
            value -= 10.0;
            if purposeful {
                value += 14.0 + self.side_controls as f64 * 2.0;
            }
            if self.view.hand.len() > play.cards.len() + 3 {
                value -= f64::from(play.strongest_component().comparison_strength()) * 0.6;
                if play.cards.iter().any(|card| card.suit().is_none()) {
                    value -= 9.0;
                }
            }
            if self.knowledge.outside_count(4) == 0 && !self.is_last(play) {
                value -= 12.0;
            }
        }
        value + self.bottom_value(play, confidence)
    }

    fn follow_score(
        &self,
        play: &ShengjiClassifiedPlay,
        trick: &ShengjiTrickRecord,
        cost: f64,
    ) -> f64 {
        let lead = &trick.plays[0].1;
        let current = &trick
            .plays
            .iter()
            .find(|(seat, _)| *seat == trick.winner)
            .expect("winner")
            .1;
        let wins = compare_for_trick(lead, current, play, self.view.trump).is_gt();
        let (winner, seat) = if wins {
            (play, self.view.player)
        } else {
            (current, trick.winner)
        };
        let before = self
            .knowledge
            .team_probability(lead, current, trick.winner, &self.remaining);
        let confidence = self
            .knowledge
            .team_probability(lead, winner, seat, &self.remaining);
        let friendly = ShengjiTeamId::of(trick.winner) == ShengjiTeamId::of(self.view.player);
        let own = points(play);
        let captured = f64::from(trick.points) + own;
        let mut value = confidence * captured * 2.2 - (1.0 - confidence) * own * 2.5
            + (confidence - before) * 16.0
            - cost * 0.7;
        if friendly && wins && confidence < before + 0.12 {
            value -= 12.0;
        }
        if friendly && !wins && confidence >= 0.85 {
            value += own * 0.8;
        }
        if !self.is_last(play) {
            let trumps = play
                .cards
                .iter()
                .filter(|card| self.view.trump.is_trump(**card))
                .count();
            value -= trumps as f64 * 1.2;
            if play.cards.iter().any(|card| card.suit().is_none()) {
                value -= 2.5;
            }
            if trumps == play.cards.len()
                && lead.category != ShengjiCategory::Trump
                && captured == 0.0
            {
                value -= 6.0;
                value += self.side_controls as f64 * (confidence - before) * 3.0;
            }
        }
        let next = if self.view.collecting_score == 0 {
            5
        } else {
            (self.view.collecting_score / self.view.rules.score_step() + 1)
                * self.view.rules.score_step()
        };
        if self.view.collecting_score + captured as u32 >= next {
            value += (confidence - before) * 18.0;
        }
        value + self.bottom_value(play, confidence)
    }

    fn is_last(&self, play: &ShengjiClassifiedPlay) -> bool {
        self.view.hand.len() == play.cards.len()
    }

    fn bottom_value(&self, play: &ShengjiClassifiedPlay, confidence: f64) -> f64 {
        if self.is_last(play) {
            let bottom = self.view.bottom.map_or_else(
                || {
                    let count: usize = (0..5).map(|cat| self.knowledge.outside_count(cat)).sum();
                    self.knowledge.outside_points() / count.max(1) as f64
                        * self.view.rules.kitty_size() as f64
                        * 0.65
                },
                |cards| cards.iter().map(|card| f64::from(card.points())).sum(),
            );
            let multiplier = self.view.trick.map_or(play.kitty_multiplier(), |trick| {
                let current = &trick
                    .plays
                    .iter()
                    .find(|(seat, _)| *seat == trick.winner)
                    .expect("winner")
                    .1;
                if compare_for_trick(&trick.plays[0].1, current, play, self.view.trump).is_gt() {
                    play.kitty_multiplier()
                } else {
                    current.kitty_multiplier()
                }
            });
            return (confidence * 2.0 - 1.0) * (24.0 + bottom * f64::from(multiplier));
        }
        if self.view.hand.len() <= 8
            && play
                .cards
                .iter()
                .any(|card| self.view.trump.is_trump(*card))
            && !self
                .view
                .hand
                .iter()
                .any(|card| self.view.trump.is_trump(*card) && !play.cards.contains(card))
        {
            return -8.0;
        }
        0.0
    }
}

fn points(play: &ShengjiClassifiedPlay) -> f64 {
    play.cards.iter().map(|card| f64::from(card.points())).sum()
}
