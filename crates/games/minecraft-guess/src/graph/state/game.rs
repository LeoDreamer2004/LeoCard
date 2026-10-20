use super::super::data::{GraphData, GraphEdge};
use super::super::sampling::Sample;
use super::combo::Combo;
use crate::{Entry, GameError, GraphGuessRecord, GraphRules};
use std::mem;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct GraphGame {
    pub(super) nodes: Vec<usize>,
    pub(super) edges: Vec<GraphEdge>,
    pub(super) revealed: Vec<bool>,
    pub(super) guesses: Vec<GraphGuessRecord>,
    pub(super) selected: usize,
    pub(super) cooldown: Duration,
    pub(super) rules: GraphRules,
    pub(super) players: Vec<u8>,
    pub(super) current: usize,
    pub(super) skipped: Vec<u8>,
    pub(super) gains: Vec<(u8, u32)>,
    pub(super) combo: Option<Combo>,
}

impl GraphGame {
    pub fn new(players: Vec<u8>, rules: GraphRules, seed: u64) -> Result<Self, GameError> {
        let sample = Sample::generate(rules, seed).ok_or(GameError::GraphUnavailable)?;
        let mut game = Self::pending(players, rules);
        game.revealed = vec![false; sample.nodes.len()];
        game.nodes = sample.nodes;
        game.edges = sample.edges;
        Ok(game)
    }

    pub fn pending(players: Vec<u8>, rules: GraphRules) -> Self {
        Self {
            players,
            rules,
            current: 0,
            skipped: Vec::new(),
            gains: Vec::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            revealed: Vec::new(),
            guesses: Vec::new(),
            selected: 0,
            cooldown: Duration::ZERO,
            combo: None,
        }
    }

    pub(super) fn entry(&self, index: usize) -> Option<&'static Entry> {
        self.nodes
            .get(index)
            .and_then(|&node| GraphData::bundled().nodes[node].entry)
    }

    pub(super) fn neighbours(&self, index: usize) -> Vec<usize> {
        let node = self.nodes[index];
        let mut neighbours: Vec<_> = self
            .edges
            .iter()
            .filter_map(|edge| {
                if edge.from == node {
                    Some(edge.to)
                } else if edge.to == node {
                    Some(edge.from)
                } else {
                    None
                }
            })
            .filter(|&other| other != node)
            .map(|other| {
                self.nodes
                    .iter()
                    .position(|&id| id == other)
                    .expect("sample endpoint")
            })
            .collect();
        neighbours.sort_unstable();
        neighbours.dedup();
        neighbours
    }

    pub fn advance_time(&mut self, elapsed: Duration) {
        self.cooldown = self.cooldown.saturating_sub(elapsed);
    }

    pub fn select(&mut self, player: u8, number: u16) -> Result<(), GameError> {
        self.check_player(player)?;
        let index = usize::from(number).wrapping_sub(1);
        if index >= self.nodes.len() {
            return Err(GameError::InvalidNode);
        }
        if index == self.selected {
            return Ok(());
        }
        self.check_combo_target(index)?;
        if !self.cooldown.is_zero() {
            return Err(GameError::NodeSwitchCoolingDown);
        }
        self.selected = index;
        self.cooldown = Duration::from_millis(600);
        Ok(())
    }

    pub fn current_player(&self) -> Option<u8> {
        (!self.nodes.is_empty() && !self.finished()).then(|| self.players[self.current])
    }

    pub(super) fn check_player(&self, player: u8) -> Result<(), GameError> {
        if self.finished() {
            return Err(GameError::Finished);
        }
        if self.current_player() != Some(player) {
            return Err(GameError::NotPlayersTurn);
        }
        Ok(())
    }

    pub fn skip(&mut self, player: u8) -> Result<(), GameError> {
        self.check_player(player)?;
        self.skipped.push(player);
        if self.skipped.len() == self.players.len() {
            self.skipped.clear();
        }
        self.end_turn();
        Ok(())
    }

    pub fn take_score_gains(&mut self) -> Vec<(u8, u32)> {
        mem::take(&mut self.gains)
    }

    /// Finish this graph without inventing guesses or awarding unanswered clues.
    pub fn reveal_all(&mut self) -> Result<(), GameError> {
        if self.nodes.is_empty() {
            return Err(GameError::GraphUnavailable);
        }
        if self.finished() {
            return Err(GameError::Finished);
        }
        self.revealed.fill(true);
        self.combo = None;
        self.skipped.clear();
        self.cooldown = Duration::ZERO;
        Ok(())
    }

    pub fn finished(&self) -> bool {
        !self.nodes.is_empty() && self.revealed.iter().all(|revealed| *revealed)
    }
}
