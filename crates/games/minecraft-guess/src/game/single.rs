use std::mem;

use super::analysis::{GuessAnalysisInput, RoundAnalysis};
use super::{
    GameError, GuessActionStatistics, GuessPlayStatistics, GuessRecord, GuessResult, RoundView,
};
use crate::{Entry, GuessMode, MinecraftGuessRuleSet};

#[derive(Clone, Debug)]
pub struct SingleGameState {
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

impl SingleGameState {
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
            completed: self.finished,
            kind: self.answer.kind,
            character_count: self.answer.name.chars().count() as u16,
            current_player: self.current_player(),
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
        (!self.finished && self.rules.mode == GuessMode::TurnBased)
            .then(|| self.players[self.current])
    }

    fn check_action(&self, player: u8) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::Finished);
        }
        if !self.players.contains(&player)
            || (self.rules.mode == GuessMode::TurnBased && self.current_player() != Some(player))
        {
            return Err(GameError::NotPlayersTurn);
        }
        Ok(())
    }

    pub fn guess(&mut self, player: u8, name_or_id: &str) -> Result<(), GameError> {
        self.check_action(player)?;
        let result = GuessResult::resolve(
            &self.answer,
            name_or_id,
            self.rules.options().allow_unknown_guesses,
        )?;
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
            mode: self.rules.mode,
            result: &result,
            previous,
            turns: self.turns,
            guess_count: self.guesses.len(),
            skipped: &self.skipped,
        });
        let first_matching_available =
            !self.guesses.iter().any(|guess| guess.result.all_matching());
        let score = match self.rules.mode {
            GuessMode::TurnBased => result.score(
                previous,
                self.turns / self.players.len(),
                first_matching_available,
            ),
            GuessMode::Race => result.race_score(first_matching_available),
            GuessMode::Graph => unreachable!("graph has independent rules"),
        };
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

    pub fn skip(&mut self, player: u8) -> Result<(), GameError> {
        self.check_action(player)?;
        if self.rules.mode == GuessMode::Race {
            return Err(GameError::CannotSkip);
        }
        self.analysis.skip(player);
        self.skipped.push(player);
        if self.skipped.len() == self.players.len() {
            self.finished = true;
        } else {
            self.next();
        }
        Ok(())
    }

    pub fn expire(&mut self) -> Result<(), GameError> {
        if self.finished {
            return Err(GameError::Finished);
        }
        self.finished = true;
        Ok(())
    }

    pub fn take_action_statistics(&mut self) -> Vec<GuessActionStatistics> {
        self.analysis.take_actions()
    }

    pub fn consistency(&self) -> impl Iterator<Item = (u8, bool)> + '_ {
        self.analysis.consistency()
    }

    pub fn statistics(&self) -> impl Iterator<Item = (u8, GuessPlayStatistics)> + '_ {
        assert!(self.finished);
        self.analysis.statistics().map(|(player, mut statistics)| {
            statistics.finish_game(
                self.answer.kind,
                self.winner == Some(player),
                self.rules.options().allow_unknown_guesses,
            );
            (player, statistics)
        })
    }

    pub fn take_score_gains(&mut self) -> Vec<(u8, u32)> {
        mem::take(&mut self.pending_gains)
    }

    fn next(&mut self) {
        self.turns += 1;
        if self.rules.mode == GuessMode::TurnBased {
            self.current = (self.current + 1) % self.players.len();
        }
    }
}
