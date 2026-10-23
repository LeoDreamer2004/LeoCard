use super::GraphGame;
use super::combo::Combo;
use crate::{GameError, GraphGuessRecord, GuessRecord, GuessResult};
use std::time::Duration;

impl GraphGame {
    pub fn guess(&mut self, player: u8, text: &str) -> Result<(), GameError> {
        self.check_player(player)?;
        if self.revealed[self.selected] {
            return Err(GameError::NodeAlreadyRevealed);
        }
        let answer = self.entry(self.selected).ok_or(GameError::CannotGuessTag)?;
        self.check_combo_target(self.selected)?;
        let result = GuessResult::resolve(answer, text, self.rules.options.allow_unknown_guesses)?;
        let number = self.selected as u16 + 1;
        if self
            .guesses
            .iter()
            .any(|record| record.node == number && record.guess.result.name() == result.name())
        {
            return Err(GameError::AlreadyGuessed);
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
        self.analysis
            .guess(self.selected, player, &result, previous);
        self.guesses.push(GraphGuessRecord {
            node: number,
            guess: GuessRecord { player, result },
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
