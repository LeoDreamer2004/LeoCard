use super::single::McGuessSingleGameState;
use super::{McGuessGameError, McGuessRoundView};
use crate::{
    McGuessActionStatistics, McGuessEntry, McGuessGraphGame, McGuessGraphView,
    McGuessHandStatistics, McGuessMode, McGuessPlayStatistics, McGuessRuleSet,
};
use leocard_game_common::PlayerId;
use std::time::Duration;

#[derive(Clone, Debug)]
enum Round {
    Single(McGuessSingleGameState),
    Graph(McGuessGraphGame),
}

/// A mode-neutral host interface; each game owns its private answers and transitions.
#[derive(Clone, Debug)]
pub struct McGuessGameState {
    round: Round,
}

impl McGuessGameState {
    pub fn new(answer: McGuessEntry, players: Vec<PlayerId>, rules: McGuessRuleSet) -> Self {
        assert_ne!(rules.mode, McGuessMode::Graph);
        Self {
            round: Round::Single(McGuessSingleGameState::new(answer, players, rules)),
        }
    }

    pub fn pending_graph(players: Vec<PlayerId>, rules: McGuessRuleSet) -> Self {
        Self {
            round: Round::Graph(McGuessGraphGame::pending(players, rules.graph)),
        }
    }

    pub fn new_graph(
        players: Vec<PlayerId>,
        rules: McGuessRuleSet,
        seed: u64,
    ) -> Result<Self, McGuessGameError> {
        assert!(!players.is_empty() && rules.is_valid() && rules.mode == McGuessMode::Graph);
        Ok(Self {
            round: Round::Graph(McGuessGraphGame::new(players, rules.graph, seed)?),
        })
    }

    pub fn view(&self) -> McGuessRoundView {
        match &self.round {
            Round::Single(game) => game.view(),
            Round::Graph(game) => game.round_view(),
        }
    }

    pub fn graph_view(&self) -> Option<McGuessGraphView> {
        match &self.round {
            Round::Single(_) => None,
            Round::Graph(game) => Some(game.view()),
        }
    }

    pub fn current_player(&self) -> Option<PlayerId> {
        match &self.round {
            Round::Single(game) => game.current_player(),
            Round::Graph(game) => game.current_player(),
        }
    }

    pub fn finished(&self) -> bool {
        match &self.round {
            Round::Single(game) => game.finished(),
            Round::Graph(game) => game.finished(),
        }
    }

    pub fn guess(&mut self, player: PlayerId, text: &str) -> Result<(), McGuessGameError> {
        match &mut self.round {
            Round::Single(game) => game.guess(player, text),
            Round::Graph(game) => game.guess(player, text),
        }
    }

    pub fn skip(&mut self, player: PlayerId) -> Result<(), McGuessGameError> {
        match &mut self.round {
            Round::Single(game) => game.skip(player),
            Round::Graph(game) => game.skip(player),
        }
    }

    pub fn expire(&mut self) -> Result<(), McGuessGameError> {
        match &mut self.round {
            Round::Single(game) => game.expire(),
            Round::Graph(_) => Err(McGuessGameError::CannotSkip),
        }
    }

    pub fn select_node(&mut self, player: PlayerId, number: u16) -> Result<(), McGuessGameError> {
        match &mut self.round {
            Round::Graph(game) => game.select(player, number),
            Round::Single(_) => Err(McGuessGameError::InvalidNode),
        }
    }

    pub fn reveal_graph(&mut self) -> Result<(), McGuessGameError> {
        match &mut self.round {
            Round::Graph(game) => game.reveal_all(),
            Round::Single(_) => Err(McGuessGameError::InvalidNode),
        }
    }

    pub fn advance_selection_time(&mut self, elapsed: Duration) {
        if let Round::Graph(game) = &mut self.round {
            game.advance_time(elapsed);
        }
    }

    pub fn take_score_gains(&mut self) -> Vec<(PlayerId, u32)> {
        match &mut self.round {
            Round::Single(game) => game.take_score_gains(),
            Round::Graph(game) => game.take_score_gains(),
        }
    }

    pub fn take_action_statistics(&mut self) -> Vec<McGuessActionStatistics> {
        match &mut self.round {
            Round::Single(game) => game.take_action_statistics(),
            Round::Graph(game) => game.take_action_statistics(),
        }
    }

    /// Graph nodes finish independently before the whole graph is complete.
    pub fn take_hand_statistics(&mut self) -> Vec<McGuessHandStatistics> {
        match &mut self.round {
            Round::Single(_) => Vec::new(),
            Round::Graph(game) => game.take_hand_statistics(),
        }
    }

    pub fn statistics(&self) -> impl Iterator<Item = (PlayerId, McGuessPlayStatistics)> {
        match &self.round {
            Round::Single(game) => game.statistics().collect::<Vec<_>>(),
            Round::Graph(game) => game.statistics().collect::<Vec<_>>(),
        }
        .into_iter()
    }

    pub fn consistency(&self) -> impl Iterator<Item = (PlayerId, bool)> {
        match &self.round {
            Round::Single(game) => game.consistency().collect::<Vec<_>>(),
            Round::Graph(_) => Vec::new(),
        }
        .into_iter()
    }
}
