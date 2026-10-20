use super::super::data::GraphData;
use super::GraphGame;
use crate::{EntryKind, GraphCombo, GraphEdgeView, GraphNodeView, GraphView, RoundView};

impl GraphGame {
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
            guesses: self
                .guesses
                .iter()
                .filter(|record| usize::from(record.node - 1) == self.selected)
                .map(|record| record.guess.clone())
                .collect(),
            skipped_players: self.skipped.clone(),
            answer: None,
            winner: None,
        }
    }

    pub fn view(&self) -> GraphView {
        let data = GraphData::bundled();
        GraphView {
            selected: self.selected as u16 + 1,
            combo: self.combo.map(|combo| GraphCombo {
                count: combo.count,
                targets: self
                    .available_nodes()
                    .into_iter()
                    .map(|index| index as u16 + 1)
                    .collect(),
            }),
            guesses: self.guesses.clone(),
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
