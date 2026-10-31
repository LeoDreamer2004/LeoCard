use super::super::data::{GraphData, GraphEdge};
use super::super::sampling::Sample;
use super::analysis::GraphAnalysis;
use super::combo::Combo;
use crate::{McGuessEntry, McGuessGameError, McGuessGraphGuessRecord, McGuessGraphRules};
use leocard_game_common::PlayerId;
use std::mem;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct McGuessGraphGame {
    pub(super) nodes: Vec<usize>,
    pub(super) edges: Vec<GraphEdge>,
    pub(super) revealed: Vec<bool>,
    pub(super) guesses: Vec<McGuessGraphGuessRecord>,
    pub(super) selected: usize,
    pub(super) cooldown: Duration,
    pub(super) rules: McGuessGraphRules,
    pub(super) players: Vec<PlayerId>,
    pub(super) current: usize,
    pub(super) skipped: Vec<PlayerId>,
    pub(super) gains: Vec<(PlayerId, u32)>,
    pub(super) combo: Option<Combo>,
    pub(super) analysis: GraphAnalysis,
}

impl McGuessGraphGame {
    pub fn new(
        players: Vec<PlayerId>,
        rules: McGuessGraphRules,
        seed: u64,
    ) -> Result<Self, McGuessGameError> {
        let sample = Sample::generate(rules, seed).ok_or(McGuessGameError::GraphUnavailable)?;
        let mut game = Self::pending(players, rules);
        game.revealed = vec![false; sample.nodes.len()];
        game.analysis = GraphAnalysis::new(&game.players, sample.nodes.len());
        game.nodes = sample.nodes;
        game.edges = sample.edges;
        Ok(game)
    }

    pub fn pending(players: Vec<PlayerId>, rules: McGuessGraphRules) -> Self {
        Self {
            analysis: GraphAnalysis::new(&players, 0),
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

    pub(super) fn entry(&self, index: usize) -> Option<&'static McGuessEntry> {
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

    pub fn select(&mut self, player: PlayerId, number: u16) -> Result<(), McGuessGameError> {
        self.check_player(player)?;
        let index = usize::from(number).wrapping_sub(1);
        if index >= self.nodes.len() {
            return Err(McGuessGameError::InvalidNode);
        }
        if index == self.selected {
            return Ok(());
        }
        self.check_combo_target(index)?;
        if !self.cooldown.is_zero() {
            return Err(McGuessGameError::NodeSwitchCoolingDown);
        }
        self.selected = index;
        self.cooldown = Duration::from_millis(600);
        Ok(())
    }

    pub fn current_player(&self) -> Option<PlayerId> {
        (!self.nodes.is_empty() && !self.finished()).then(|| self.players[self.current])
    }

    pub(super) fn check_player(&self, player: PlayerId) -> Result<(), McGuessGameError> {
        if self.finished() {
            return Err(McGuessGameError::Finished);
        }
        if self.current_player() != Some(player) {
            return Err(McGuessGameError::NotPlayersTurn);
        }
        Ok(())
    }

    pub fn skip(&mut self, player: PlayerId) -> Result<(), McGuessGameError> {
        self.check_player(player)?;
        let node = (self.entry(self.selected).is_some() && !self.revealed[self.selected])
            .then_some(self.selected);
        self.analysis.skip(node, player);
        self.skipped.push(player);
        if self.skipped.len() == self.players.len() {
            self.reveal_all()?;
            self.analysis.finish();
            return Ok(());
        }
        self.end_turn();
        Ok(())
    }

    pub fn take_score_gains(&mut self) -> Vec<(PlayerId, u32)> {
        mem::take(&mut self.gains)
    }

    /// Finish this graph without inventing guesses or awarding unanswered clues.
    pub fn reveal_all(&mut self) -> Result<(), McGuessGameError> {
        if self.nodes.is_empty() {
            return Err(McGuessGameError::GraphUnavailable);
        }
        if self.finished() {
            return Err(McGuessGameError::Finished);
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
