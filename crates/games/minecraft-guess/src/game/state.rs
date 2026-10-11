use std::mem;

use super::GuessActionStatistics;
use super::analysis::{GuessAnalysisInput, RoundAnalysis};
use super::{GameError, GuessRecord, GuessResult, RoundView};
use crate::{Catalogue, Entry, MAX_GUESS_CHARACTERS, MinecraftGuessRuleSet};

#[derive(Clone, Debug)]
pub struct GameState {
    answer: Entry,
    players: Vec<u8>,
    current: usize,
    turns: usize,
    pending_gains: Vec<(u8, u32)>,
    guesses: Vec<GuessRecord>,
    skipped: Vec<u8>,
    finished: bool,
    winner: Option<u8>,
    rules: MinecraftGuessRuleSet,
    analysis: RoundAnalysis,
}

impl GameState {
    pub fn new(answer: Entry, players: Vec<u8>, rules: MinecraftGuessRuleSet) -> Self {
        assert!(!players.is_empty());
        assert!(rules.is_valid());
        Self {
            analysis: RoundAnalysis::new(&players),
            answer,
            players,
            current: 0,
            turns: 0,
            pending_gains: Vec::new(),
            guesses: Vec::new(),
            skipped: Vec::new(),
            finished: false,
            winner: None,
            rules,
        }
    }

    pub fn view(&self) -> RoundView {
        RoundView {
            kind: self.answer.kind,
            character_count: self.answer.name.chars().count() as u16,
            current_player: (!self.finished).then(|| self.players[self.current]),
            guesses: self.guesses.clone(),
            skipped_players: self.skipped.clone(),
            answer: self.finished.then(|| self.answer.clone()),
            winner: self.winner,
        }
    }

    pub fn finished(&self) -> bool {
        self.finished
    }

    pub fn current_player(&self) -> Option<u8> {
        (!self.finished).then(|| self.players[self.current])
    }

    fn check_turn(&self, player: u8) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::Finished);
        }
        if self.current_player() != Some(player) {
            return Err(GameError::NotPlayersTurn);
        }
        Ok(())
    }

    pub fn guess(&mut self, player: u8, name_or_id: &str) -> Result<(), GameError> {
        self.check_turn(player)?;
        let result = self.resolve_guess(name_or_id)?;
        if self
            .guesses
            .iter()
            .any(|guess| guess.result.name() == result.name())
        {
            return Err(GameError::AlreadyGuessed);
        }
        let previous = self
            .guesses
            .iter()
            .rev()
            .find_map(|guess| guess.result.closeness());
        self.analysis.guess(GuessAnalysisInput {
            player,
            result: &result,
            previous,
            turns: self.turns,
            guess_count: self.guesses.len(),
            skipped: &self.skipped,
        });
        let score = result.score(previous, self.turns / self.players.len());
        for points in [score.during_guess, score.after_answer] {
            if points > 0 {
                self.pending_gains.push((player, points));
            }
        }
        let correct = result.is_correct();
        self.guesses.push(GuessRecord { player, result });
        self.skipped.clear();
        if correct {
            self.finished = true;
            self.winner = Some(player);
        } else {
            self.next();
        }
        Ok(())
    }

    fn resolve_guess(&self, name_or_id: &str) -> Result<GuessResult, GameError> {
        let name = name_or_id.trim();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(GameError::InvalidGuess);
        }
        let entry = Catalogue::bundled().find(self.answer.kind, name);
        // Canonical IDs are used by candidate buttons and bots, not as displayed names.
        if !entry.is_some_and(|entry| entry.id == name)
            && name.chars().count() > MAX_GUESS_CHARACTERS
        {
            return Err(GameError::GuessTooLong);
        }
        match entry {
            Some(entry) => Ok(GuessResult::Known {
                entry: entry.clone(),
                correct: entry.id == self.answer.id,
                matches: entry
                    .attributes
                    .iter()
                    .zip(&self.answer.attributes)
                    .map(|(ours, theirs)| ours.feedback(theirs))
                    .collect(),
            }),
            None if self.rules.allow_unknown_guesses => Ok(GuessResult::Unknown {
                name: name.to_owned(),
            }),
            None => Err(GameError::UnknownEntry),
        }
    }

    pub fn skip(&mut self, player: u8) -> Result<(), GameError> {
        self.check_turn(player)?;
        self.analysis.skip(player);
        self.skipped.push(player);
        if self.skipped.len() == self.players.len() {
            self.finished = true;
        } else {
            self.next();
        }
        Ok(())
    }

    pub fn take_action_statistics(&mut self) -> Vec<GuessActionStatistics> {
        self.analysis.take_actions()
    }

    pub fn consistency(&self) -> impl Iterator<Item = (u8, bool)> + '_ {
        self.analysis.consistency()
    }

    pub fn take_score_gains(&mut self) -> Vec<(u8, u32)> {
        mem::take(&mut self.pending_gains)
    }

    fn next(&mut self) {
        self.turns += 1;
        self.current = (self.current + 1) % self.players.len();
    }
}
