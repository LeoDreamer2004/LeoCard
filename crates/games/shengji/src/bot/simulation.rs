use super::ShengjiBotObservation;
use super::evaluation::{Evaluation, PositionView};
use super::knowledge::PublicKnowledge;
use super::sampling::SampledDeal;
use crate::{
    ShengjiCard, ShengjiClassifiedPlay, ShengjiRuleSet, ShengjiTeamId, ShengjiTrickRecord,
    ShengjiTrump, compare_for_trick,
};
use leocard_game_common::{PlayerId, SearchBudget};

#[derive(Clone)]
pub(super) struct Simulation {
    pub hands: Vec<Vec<ShengjiCard>>,
    pub trick: Option<ShengjiTrickRecord>,
    pub actor: PlayerId,
    pub collecting_score: u32,
    pub collecting_team: ShengjiTeamId,
    pub rules: ShengjiRuleSet,
    pub trump: ShengjiTrump,
    pub public: PublicKnowledge,
    bottom: Vec<ShengjiCard>,
    observer: PlayerId,
    known_bottom: Option<Vec<ShengjiCard>>,
}

impl Simulation {
    pub fn new(
        observation: &ShengjiBotObservation,
        deal: SampledDeal,
        public: &PublicKnowledge,
    ) -> Self {
        Self {
            hands: deal.hands,
            bottom: deal.bottom,
            trick: observation.trick.clone(),
            actor: observation.player,
            collecting_score: observation.collecting_score,
            collecting_team: observation.collecting_team,
            rules: observation.rules,
            trump: observation.trump,
            public: public.clone(),
            observer: observation.player,
            known_bottom: observation.bottom.clone(),
        }
    }

    pub fn view<'a>(&'a self, lengths: &'a [usize]) -> PositionView<'a> {
        PositionView {
            player: self.actor,
            hand: &self.hands[usize::from(self.actor.0)],
            trump: self.trump,
            rules: self.rules,
            trick: self.trick.as_ref(),
            public: &self.public,
            lengths,
            collecting_score: self.collecting_score,
            // Each rollout seat sees only its own private information. Sampled
            // holdings of other seats never enter its heuristic observation.
            bottom: (self.actor == self.observer)
                .then_some(self.known_bottom.as_deref())
                .flatten(),
        }
    }

    pub fn play(&mut self, play: ShengjiClassifiedPlay) {
        let player = self.actor;
        let trick = self.trick.get_or_insert_with(|| ShengjiTrickRecord {
            leader: player,
            winner: player,
            plays: Vec::new(),
            points: 0,
        });
        if !trick.plays.is_empty() {
            let current = &trick
                .plays
                .iter()
                .find(|(seat, _)| *seat == trick.winner)
                .expect("winner")
                .1;
            if compare_for_trick(&trick.plays[0].1, current, &play, self.trump).is_gt() {
                trick.winner = player;
            }
        }
        self.public.record(
            player,
            &play,
            trick.plays.first().map_or(&play, |(_, lead)| lead),
            self.trump,
        );
        self.hands[usize::from(player.0)].retain(|card| !play.cards.contains(card));
        trick.points += play.cards.iter().map(|card| card.points()).sum::<u16>();
        trick.plays.push((player, play));
        if trick.plays.len() == usize::from(self.rules.player_count()) {
            let trick = self.trick.take().expect("complete trick");
            if ShengjiTeamId::of(trick.winner) == self.collecting_team {
                self.collecting_score += u32::from(trick.points);
                if self.finished() {
                    let winner = &trick
                        .plays
                        .iter()
                        .find(|(seat, _)| *seat == trick.winner)
                        .expect("winner")
                        .1;
                    self.collecting_score += self
                        .bottom
                        .iter()
                        .map(|card| u32::from(card.points()))
                        .sum::<u32>()
                        * winner.kitty_multiplier();
                }
            }
            self.actor = trick.winner;
        } else {
            self.actor = PlayerId((player.0 + 1) % self.rules.player_count());
        }
    }

    pub fn finished(&self) -> bool {
        self.hands.iter().all(Vec::is_empty)
    }

    pub fn utility(&self, player: PlayerId) -> f64 {
        let score = self.collecting_score;
        let step = self.rules.score_step();
        let levels = if score == 0 {
            -3.0
        } else if score < step {
            -2.0
        } else if score < self.rules.takeover_score() {
            -1.0
        } else {
            1.0 + f64::from((score - self.rules.takeover_score()) / step)
        };
        let sign = if ShengjiTeamId::of(player) == self.collecting_team {
            1.0
        } else {
            -1.0
        };
        sign * (levels * f64::from(step) + f64::from(score) * 0.15)
    }

    pub fn rollout(mut self, player: PlayerId, budget: &SearchBudget) -> Option<f64> {
        // Follow the actual strategy to the end. This values long-suit plans
        // and bottom control, instead of discounting everything after 3 tricks.
        while !self.finished() {
            if budget.exhausted() {
                return None;
            }
            let lengths: Vec<_> = self.hands.iter().map(Vec::len).collect();
            let play = Evaluation::new(self.view(&lengths)).ranked(1).remove(0).0;
            self.play(play);
        }
        Some(self.utility(player))
    }
}
