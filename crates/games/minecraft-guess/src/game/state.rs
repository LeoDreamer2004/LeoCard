use super::single::SingleGameState;
use super::{GameError, GuessActionStatistics, GuessPlayStatistics, RoundView};
use crate::{Entry, GraphGame, GraphView, GuessMode, MinecraftGuessRuleSet};
use std::time::Duration;

#[derive(Clone, Debug)]
enum Round {
    Single(SingleGameState),
    Graph(GraphGame),
}

/// A mode-neutral host interface; each game owns its private answers and transitions.
#[derive(Clone, Debug)]
pub struct GameState {
    round: Round,
}

impl GameState {
    pub fn new(answer: Entry, players: Vec<u8>, rules: MinecraftGuessRuleSet) -> Self {
        assert_ne!(rules.mode, GuessMode::Graph);
        Self {
            round: Round::Single(SingleGameState::new(answer, players, rules)),
        }
    }

    pub fn new_graph(
        players: Vec<u8>,
        rules: MinecraftGuessRuleSet,
        seed: u64,
    ) -> Result<Self, GameError> {
        assert!(!players.is_empty() && rules.is_valid() && rules.mode == GuessMode::Graph);
        Ok(Self {
            round: Round::Graph(GraphGame::new(players, rules.graph, seed)?),
        })
    }

    pub fn view(&self) -> RoundView {
        match &self.round {
            Round::Single(game) => game.view(),
            Round::Graph(game) => game.round_view(),
        }
    }

    pub fn graph_view(&self) -> Option<GraphView> {
        match &self.round {
            Round::Single(_) => None,
            Round::Graph(game) => Some(game.view()),
        }
    }

    pub fn current_player(&self) -> Option<u8> {
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

    pub fn guess(&mut self, player: u8, text: &str) -> Result<(), GameError> {
        match &mut self.round {
            Round::Single(game) => game.guess(player, text),
            Round::Graph(game) => game.guess(player, text),
        }
    }

    pub fn skip(&mut self, player: u8) -> Result<(), GameError> {
        match &mut self.round {
            Round::Single(game) => game.skip(player),
            Round::Graph(game) => game.skip(player),
        }
    }

    pub fn expire(&mut self) -> Result<(), GameError> {
        match &mut self.round {
            Round::Single(game) => game.expire(),
            Round::Graph(_) => Err(GameError::CannotSkip),
        }
    }

    pub fn select_node(&mut self, player: u8, number: u16) -> Result<(), GameError> {
        match &mut self.round {
            Round::Graph(game) => game.select(player, number),
            Round::Single(_) => Err(GameError::InvalidNode),
        }
    }

    pub fn advance_selection_time(&mut self, elapsed: Duration) {
        if let Round::Graph(game) = &mut self.round {
            game.advance_time(elapsed);
        }
    }

    pub fn take_score_gains(&mut self) -> Vec<(u8, u32)> {
        match &mut self.round {
            Round::Single(game) => game.take_score_gains(),
            Round::Graph(game) => game.take_score_gains(),
        }
    }

    pub fn take_action_statistics(&mut self) -> Vec<GuessActionStatistics> {
        match &mut self.round {
            Round::Single(game) => game.take_action_statistics(),
            Round::Graph(_) => Vec::new(),
        }
    }

    pub fn statistics(&self) -> impl Iterator<Item = (u8, GuessPlayStatistics)> {
        match &self.round {
            Round::Single(game) => game.statistics().collect::<Vec<_>>(),
            Round::Graph(_) => Vec::new(),
        }
        .into_iter()
    }

    pub fn consistency(&self) -> impl Iterator<Item = (u8, bool)> {
        match &self.round {
            Round::Single(game) => game.consistency().collect::<Vec<_>>(),
            Round::Graph(_) => Vec::new(),
        }
        .into_iter()
    }
}
