use super::{McGuessGameError, McGuessRecord, McGuessResult, McGuessRoundView};
use crate::analysis::{GuessAnalysisInput, RoundAnalysis};
use crate::{
    McGuessActionStatistics, McGuessEntry, McGuessMode, McGuessPlayStatistics, McGuessRuleSet,
};
use leocard_game_common::PlayerId;
use std::mem;

#[derive(Clone, Debug)]
pub struct McGuessSingleGameState {
    answer: McGuessEntry,
    players: Vec<PlayerId>,
    current: usize,
    turns: usize,
    pending_gains: Vec<(PlayerId, u32)>,
    guesses: Vec<McGuessRecord>,
    skipped: Vec<PlayerId>,
    finished: bool,
    winner: Option<PlayerId>,
    rules: McGuessRuleSet,
    analysis: RoundAnalysis,
}

impl McGuessSingleGameState {
    pub fn new(answer: McGuessEntry, players: Vec<PlayerId>, rules: McGuessRuleSet) -> Self {
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

    pub fn view(&self) -> McGuessRoundView {
        McGuessRoundView {
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

    pub fn current_player(&self) -> Option<PlayerId> {
        (!self.finished && self.rules.mode == McGuessMode::TurnBased)
            .then(|| self.players[self.current])
    }

    fn check_action(&self, player: PlayerId) -> Result<(), McGuessGameError> {
        if self.finished {
            return Err(McGuessGameError::Finished);
        }
        if !self.players.contains(&player)
            || (self.rules.mode == McGuessMode::TurnBased && self.current_player() != Some(player))
        {
            return Err(McGuessGameError::NotPlayersTurn);
        }
        Ok(())
    }

    pub fn guess(&mut self, player: PlayerId, name_or_id: &str) -> Result<(), McGuessGameError> {
        self.check_action(player)?;
        let result = McGuessResult::resolve(
            &self.answer,
            name_or_id,
            self.rules.options().allow_unknown_guesses,
        )?;
        if self
            .guesses
            .iter()
            .any(|guess| guess.result.name() == result.name())
        {
            return Err(McGuessGameError::AlreadyGuessed);
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
            character_count: Some(self.answer.name.chars().count()),
            skipped: &self.skipped,
        });
        let first_matching_available =
            !self.guesses.iter().any(|guess| guess.result.all_matching());
        let score = match self.rules.mode {
            McGuessMode::TurnBased => result.score(
                previous,
                self.turns / self.players.len(),
                first_matching_available,
            ),
            McGuessMode::Race => result.race_score(first_matching_available),
            McGuessMode::Graph => unreachable!("graph has independent rules"),
        };
        for points in [score.during_guess, score.after_answer] {
            if points > 0 {
                self.pending_gains.push((player, points));
            }
        }
        let correct = result.is_correct();
        self.guesses.push(McGuessRecord { player, result });
        self.skipped.clear();
        if correct {
            self.finished = true;
            self.winner = Some(player);
        } else {
            self.next();
        }
        Ok(())
    }

    pub fn skip(&mut self, player: PlayerId) -> Result<(), McGuessGameError> {
        self.check_action(player)?;
        if self.rules.mode == McGuessMode::Race {
            return Err(McGuessGameError::CannotSkip);
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

    pub fn expire(&mut self) -> Result<(), McGuessGameError> {
        if self.finished {
            return Err(McGuessGameError::Finished);
        }
        self.finished = true;
        Ok(())
    }

    pub fn take_action_statistics(&mut self) -> Vec<McGuessActionStatistics> {
        self.analysis.take_actions()
    }

    pub fn consistency(&self) -> impl Iterator<Item = (PlayerId, bool)> + '_ {
        self.analysis.consistency()
    }

    pub fn statistics(&self) -> impl Iterator<Item = (PlayerId, McGuessPlayStatistics)> + '_ {
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

    pub fn take_score_gains(&mut self) -> Vec<(PlayerId, u32)> {
        mem::take(&mut self.pending_gains)
    }

    fn next(&mut self) {
        self.turns += 1;
        if self.rules.mode == McGuessMode::TurnBased {
            self.current = (self.current + 1) % self.players.len();
        }
    }
}
