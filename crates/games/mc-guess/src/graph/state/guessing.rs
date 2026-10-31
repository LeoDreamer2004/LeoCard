use super::McGuessGraphGame;
use super::analysis::GraphGuessInput;
use super::combo::Combo;
use crate::{McGuessGameError, McGuessGraphGuessRecord, McGuessRecord, McGuessResult};
use leocard_game_common::PlayerId;
use std::time::Duration;

impl McGuessGraphGame {
    pub fn guess(&mut self, player: PlayerId, text: &str) -> Result<(), McGuessGameError> {
        self.check_player(player)?;
        if self.revealed[self.selected] {
            return Err(McGuessGameError::NodeAlreadyRevealed);
        }
        let answer = self
            .entry(self.selected)
            .ok_or(McGuessGameError::CannotGuessTag)?;
        self.check_combo_target(self.selected)?;
        let result =
            McGuessResult::resolve(answer, text, self.rules.options.allow_unknown_guesses)?;
        let number = self.selected as u16 + 1;
        if self
            .guesses
            .iter()
            .any(|record| record.node == number && record.guess.result.name() == result.name())
        {
            return Err(McGuessGameError::AlreadyGuessed);
        }
        let correct = result.is_correct();
        let combo = if correct {
            self.combo.map_or(1, |combo| combo.count + 1)
        } else {
            0
        };
        let points = self.score_guess(&result, combo);
        let previous = self
            .guesses
            .iter()
            .rev()
            .filter(|record| record.node == number)
            .find_map(|record| record.guess.result.closeness());
        let neighbours = self.neighbours(self.selected);
        self.analysis.guess(GraphGuessInput {
            index: self.selected,
            player,
            result: &result,
            previous,
            character_count: self
                .rules
                .show_character_count
                .then_some(answer.name.chars().count()),
            combo,
            points,
            only_tag_neighbours: !neighbours.is_empty()
                && neighbours.iter().all(|&index| self.entry(index).is_none()),
            entry_count: (0..self.nodes.len())
                .filter(|&index| self.entry(index).is_some())
                .count(),
        });
        self.guesses.push(McGuessGraphGuessRecord {
            node: number,
            guess: McGuessRecord { player, result },
            combo,
        });
        if correct {
            self.revealed[self.selected] = true;
            self.resolve_tags();
            self.combo = (!self.finished()).then_some(Combo {
                anchor: self.selected,
                count: combo,
            });
            self.cooldown = Duration::ZERO;
        } else {
            self.end_turn();
        }
        if points > 0 {
            self.gains.push((player, points));
        }
        self.skipped.clear();
        if self.finished() {
            self.analysis.finish();
        }
        Ok(())
    }

    fn resolve_tags(&mut self) {
        for index in 0..self.nodes.len() {
            if self.entry(index).is_some() || self.revealed[index] {
                continue;
            }
            let neighbours = self.neighbours(index);
            if neighbours.len() >= 2 && neighbours.iter().all(|&other| self.revealed[other]) {
                self.revealed[index] = true;
            }
        }
    }
}
