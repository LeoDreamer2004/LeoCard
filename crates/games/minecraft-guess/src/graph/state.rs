use super::data::GraphData;
use super::sampling::Sample;
use super::{GraphEdgeView, GraphNodeView, GraphView};
use crate::{Entry, EntryKind, GameError, GraphRules, GuessRecord, GuessResult, RoundView};
use std::mem;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct GraphGame {
    nodes: Vec<usize>,
    edges: Vec<super::data::GraphEdge>,
    revealed: Vec<bool>,
    histories: Vec<Vec<GuessRecord>>,
    selected: usize,
    cooldown: Duration,
    rules: GraphRules,
    players: Vec<u8>,
    current: usize,
    skipped: Vec<u8>,
    gains: Vec<(u8, u32)>,
}

impl GraphGame {
    pub fn new(players: Vec<u8>, rules: GraphRules, seed: u64) -> Result<Self, GameError> {
        let sample = Sample::generate(usize::from(rules.nodes), rules.include_tags, seed)
            .ok_or(GameError::GraphUnavailable)?;
        let count = sample.nodes.len();
        Ok(Self {
            players,
            current: 0,
            skipped: Vec::new(),
            gains: Vec::new(),
            nodes: sample.nodes,
            edges: sample.edges,
            revealed: vec![false; count],
            histories: vec![Vec::new(); count],
            selected: 0,
            cooldown: Duration::ZERO,
            rules,
        })
    }

    fn entry(&self, index: usize) -> Option<&'static Entry> {
        GraphData::bundled().nodes[self.nodes[index]].entry
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
        if !self.cooldown.is_zero() {
            return Err(GameError::NodeSwitchCoolingDown);
        }
        self.selected = index;
        self.cooldown = Duration::from_millis(600);
        Ok(())
    }

    pub fn guess(&mut self, player: u8, text: &str) -> Result<(), GameError> {
        self.check_player(player)?;
        if self.revealed[self.selected] {
            return Err(GameError::NodeAlreadyRevealed);
        }
        let answer = self.entry(self.selected).ok_or(GameError::CannotGuessTag)?;
        let result = GuessResult::resolve(answer, text, self.rules.options.allow_unknown_guesses)?;
        let history = &mut self.histories[self.selected];
        if history
            .iter()
            .any(|guess| guess.result.name() == result.name())
        {
            return Err(GameError::AlreadyGuessed);
        }
        let correct = result.is_correct();
        history.push(GuessRecord { player, result });
        if correct {
            self.revealed[self.selected] = true;
            self.resolve_tags();
        }
        if correct {
            self.gains.push((player, 5));
        }
        self.skipped.clear();
        self.current = (self.current + 1) % self.players.len();
        Ok(())
    }

    fn resolve_tags(&mut self) {
        for index in 0..self.nodes.len() {
            if self.entry(index).is_some() || self.revealed[index] {
                continue;
            }
            let node = self.nodes[index];
            let neighbours: Vec<_> = self
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
                .collect();
            if neighbours.len() >= 2
                && neighbours.iter().all(|other| {
                    self.revealed[self
                        .nodes
                        .iter()
                        .position(|id| id == other)
                        .expect("sample endpoint")]
                })
            {
                self.revealed[index] = true;
            }
        }
    }

    pub fn current_player(&self) -> Option<u8> {
        (!self.finished()).then(|| self.players[self.current])
    }

    fn check_player(&self, player: u8) -> Result<(), GameError> {
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
        self.current = (self.current + 1) % self.players.len();
        Ok(())
    }

    pub fn take_score_gains(&mut self) -> Vec<(u8, u32)> {
        mem::take(&mut self.gains)
    }

    pub fn finished(&self) -> bool {
        self.revealed.iter().all(|revealed| *revealed)
    }

    pub fn round_view(&self) -> RoundView {
        let entry = self.entry(self.selected);
        RoundView {
            completed: self.finished(),
            kind: entry.map_or(EntryKind::Block, |entry| entry.kind),
            character_count: if self.rules.show_character_count {
                entry.map_or(0, |e| e.name.chars().count() as u16)
            } else {
                0
            },
            current_player: self.current_player(),
            guesses: self.histories[self.selected].clone(),
            skipped_players: self.skipped.clone(),
            answer: None,
            winner: None,
        }
    }

    pub fn view(&self) -> GraphView {
        let data = GraphData::bundled();
        GraphView {
            selected: self.selected as u16 + 1,
            nodes: self
                .nodes
                .iter()
                .enumerate()
                .map(|(index, &id)| {
                    let node = &data.nodes[id];
                    GraphNodeView {
                        number: index as u16 + 1,
                        kind: node.entry.map(|entry| entry.kind),
                        character_count: (self.rules.show_character_count && !self.revealed[index])
                            .then(|| node.entry.map(|e| e.name.chars().count() as u16))
                            .flatten(),
                        entry: self.revealed[index]
                            .then_some(node.entry)
                            .flatten()
                            .cloned(),
                        tag_name: (self.revealed[index] && node.entry.is_none())
                            .then(|| node.name.clone()),
                    }
                })
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|edge| {
                    let from = self
                        .nodes
                        .iter()
                        .position(|&id| id == edge.from)
                        .expect("sample source");
                    let to = self
                        .nodes
                        .iter()
                        .position(|&id| id == edge.to)
                        .expect("sample target");
                    let active = self.revealed[from] && self.revealed[to];
                    let directed = (active || self.rules.show_directions).then_some(edge.directed);
                    let (source, target) = if directed.is_none() {
                        (from.min(to), from.max(to))
                    } else {
                        (from, to)
                    };
                    GraphEdgeView {
                        source: source as u16 + 1,
                        target: target as u16 + 1,
                        relation: if active {
                            edge.kind
                        } else {
                            edge.kind.coarse()
                        },
                        active,
                        directed,
                    }
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests;
