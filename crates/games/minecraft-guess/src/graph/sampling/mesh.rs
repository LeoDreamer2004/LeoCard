use super::super::GraphRelation;
use super::super::data::GraphEdge;
use super::index::GraphIndex;
use crate::EntryKind;
use std::collections::HashMap;

pub(in super::super) struct Sample {
    pub nodes: Vec<usize>,
    pub edges: Vec<GraphEdge>,
}

pub(super) struct Topology {
    slots: Vec<usize>,
    neighbours: Vec<[usize; 4]>,
    pub kinds: HashMap<Option<EntryKind>, usize>,
    pub relations: HashMap<GraphRelation, usize>,
    pub tags: usize,
}

impl Sample {
    pub(super) fn append(&mut self, path: &[usize], edges: Vec<GraphEdge>) {
        for &node in path {
            if !self.nodes.contains(&node) {
                self.nodes.push(node);
            }
        }
        self.edges.extend(edges);
    }
}

impl Topology {
    pub fn new(index: &GraphIndex, sample: &Sample) -> Self {
        let mut slots = vec![usize::MAX; index.data.nodes.len()];
        let mut kinds = HashMap::new();
        for (slot, &node) in sample.nodes.iter().enumerate() {
            slots[node] = slot;
            *kinds
                .entry(index.data.nodes[node].entry.map(|entry| entry.kind))
                .or_default() += 1;
        }
        let mut neighbours = vec![[usize::MAX; 4]; sample.nodes.len()];
        let mut relations = HashMap::new();
        for edge in &sample.edges {
            insert_neighbour(&mut neighbours[slots[edge.from]], edge.to);
            insert_neighbour(&mut neighbours[slots[edge.to]], edge.from);
            *relations.entry(edge.kind).or_default() += 1;
        }
        Self {
            slots,
            neighbours,
            tags: kinds.get(&None).copied().unwrap_or(0),
            kinds,
            relations,
        }
    }

    pub fn contains(&self, node: usize) -> bool {
        self.slots[node] != usize::MAX
    }

    pub fn degree(&self, node: usize) -> usize {
        if self.contains(node) {
            self.neighbours[self.slots[node]]
                .iter()
                .filter(|&&other| other != usize::MAX)
                .count()
        } else {
            0
        }
    }

    pub fn twins_ok(&self, path: &[usize]) -> bool {
        // Only a shortlisted path needs complete topology validation.
        let existing = self.neighbours.len();
        let mut neighbours = self.neighbours.clone();
        neighbours.extend((1..path.len() - 1).map(|_| [usize::MAX; 4]));
        let slot = |node| {
            if self.contains(node) {
                self.slots[node]
            } else {
                existing
                    + path[1..path.len() - 1]
                        .iter()
                        .position(|&i| i == node)
                        .unwrap()
            }
        };
        for pair in path.windows(2) {
            insert_neighbour(&mut neighbours[slot(pair[0])], pair[1]);
            insert_neighbour(&mut neighbours[slot(pair[1])], pair[0]);
        }
        let mut signatures = HashMap::<_, u8>::new();
        for mut neighbours in neighbours {
            neighbours.sort_unstable();
            let count = signatures.entry(neighbours).or_default();
            *count += 1;
            if *count > 2 {
                return false;
            }
        }
        true
    }

    pub fn quality(&self, sample: &Sample) -> f32 {
        entropy(self.kinds.values().copied().collect()) * 5.0
            + entropy(self.relations.values().copied().collect()) * 4.0
            + (sample.edges.len() - sample.nodes.len() + 1) as f32 * 0.3
    }
}

fn insert_neighbour(neighbours: &mut [usize; 4], node: usize) {
    *neighbours
        .iter_mut()
        .find(|other| **other == usize::MAX)
        .expect("mesh degree is at most four") = node;
}

fn entropy(mut counts: Vec<usize>) -> f32 {
    counts.sort_unstable();
    let total = counts.iter().sum::<usize>() as f32;
    counts
        .into_iter()
        .map(|n| {
            let p = n as f32 / total;
            -p * p.log2()
        })
        .sum()
}
