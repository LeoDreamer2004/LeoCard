use super::super::data::GraphData;
use super::GraphGame;
use crate::{EntryKind, GuessResult, MatchQuality};

const OTHER_KIND_SCORE_MULTIPLIER: f64 = 2.0 / 3.0;
const PROFESSION_SCORE_MULTIPLIER: f64 = 0.5;
const REPEAT_GUESS_SCORE_MULTIPLIER: f64 = 0.95;

struct NodeValue {
    full_degree: usize,
    sub_degree: usize,
    local_revealed: usize,
    total: usize,
    revealed: usize,
}

impl NodeValue {
    fn before_guess(game: &GraphGame) -> Self {
        let neighbours = game.neighbours(game.selected);
        Self {
            full_degree: GraphData::bundled().degree(game.nodes[game.selected]),
            sub_degree: neighbours.len(),
            local_revealed: neighbours
                .iter()
                .filter(|&&index| game.revealed[index])
                .count(),
            // Tags contribute to topology and information, but never earn points.
            total: game.nodes.len(),
            revealed: game.revealed.iter().filter(|&&revealed| revealed).count(),
        }
    }

    fn score(&self, combo: u16) -> f64 {
        let rare =
            1.0 + 12.0 / (self.full_degree as f64 + 10.0) + 4.0 / (self.sub_degree as f64 + 5.0);
        let local_blind =
            (self.sub_degree - self.local_revealed) as f64 / (self.sub_degree as f64 + 2.0);
        let global_blind = (self.total - self.revealed) as f64 / self.total as f64;
        let blind = 1.0 + 0.7 * local_blind + 0.3 * global_blind;
        let combo = if combo < 2 {
            1.0
        } else {
            (0.6 + 0.1 * f64::from(combo - 2)).min(1.2)
        };
        1000.0 * rare * blind * combo
    }
}

impl GraphGame {
    pub(super) fn score_guess(&self, result: &GuessResult, combo: u16) -> u32 {
        let GuessResult::Known { entry, matches, .. } = result else {
            return 0;
        };
        let (attempts, has_green) = self
            .guesses
            .iter()
            .filter(|record| usize::from(record.node - 1) == self.selected)
            .fold((0, false), |(attempts, has_green), record| {
                (
                    attempts + 1,
                    has_green || record.guess.result.all_matching(),
                )
            });
        let points = if result.is_correct() {
            // Answer rewards remain independent of earlier off-answer green rows.
            NodeValue::before_guess(self).score(combo) * 2.0
        } else if result.all_matching() {
            // An off-answer green row ends the turn, so neither reward uses a combo.
            NodeValue::before_guess(self).score(1) * if has_green { 0.3 } else { 1.0 }
        } else {
            let exact = matches
                .iter()
                .filter(|feedback| feedback.quality == MatchQuality::Exact)
                .count();
            let ratio = if matches.is_empty() {
                0.0
            } else {
                exact as f64 / matches.len() as f64
            };
            (20.0 + 30.0 * ratio) * 5.0
        };
        let multiplier = match entry.kind {
            EntryKind::Block | EntryKind::Item => 1.0,
            EntryKind::VillagerProfession => PROFESSION_SCORE_MULTIPLIER,
            _ => OTHER_KIND_SCORE_MULTIPLIER,
        };
        (points * multiplier * REPEAT_GUESS_SCORE_MULTIPLIER.powi(attempts)).round() as u32
    }
}

#[cfg(test)]
mod tests;
