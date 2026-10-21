use super::super::GraphRelation;
use super::super::data::GraphEdge;
use super::diversity::Diversity;
use super::index::GraphIndex;
use super::temperature::Temperature;
use crate::EntryKind;
use std::collections::{HashMap, VecDeque};

pub(in super::super) struct Sample {
    pub nodes: Vec<usize>,
    pub edges: Vec<GraphEdge>,
    pub cycles: Vec<usize>,
}

pub(super) struct Topology {
    slots: Vec<usize>,
    neighbours: Vec<Vec<usize>>,
    distances: Vec<Vec<usize>>,
    pub kinds: HashMap<Option<EntryKind>, usize>,
    pub relations: HashMap<GraphRelation, usize>,
    pub tags: usize,
    pub leaves: usize,
    pub diversity: Diversity,
}

impl Sample {
    pub(super) fn append(&mut self, path: &[usize], edges: Vec<GraphEdge>, cycle: Option<usize>) {
        for &node in path {
            if !self.nodes.contains(&node) {
                self.nodes.push(node);
            }
        }
        self.edges.extend(edges);
        self.cycles.extend(cycle);
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
        let mut neighbours = vec![Vec::new(); sample.nodes.len()];
        let mut relations = HashMap::new();
        for edge in &sample.edges {
            neighbours[slots[edge.from]].push(edge.to);
            neighbours[slots[edge.to]].push(edge.from);
            *relations.entry(edge.kind).or_default() += 1;
        }
        let mut distances = vec![vec![usize::MAX; sample.nodes.len()]; sample.nodes.len()];
        for (start, row) in distances.iter_mut().enumerate() {
            row[start] = 0;
            let mut queue = VecDeque::from([start]);
            while let Some(slot) = queue.pop_front() {
                for &other in &neighbours[slot] {
                    let next = slots[other];
                    if row[next] == usize::MAX {
                        row[next] = row[slot] + 1;
                        queue.push_back(next);
                    }
                }
            }
        }
        Self {
            diversity: index.diversity.census(sample.nodes.iter().copied()),
            slots,
            distances,
            tags: kinds.get(&None).copied().unwrap_or(0),
            leaves: neighbours.iter().filter(|others| others.len() == 1).count(),
            neighbours,
            kinds,
            relations,
        }
    }

    pub fn contains(&self, node: usize) -> bool {
        self.slots[node] != usize::MAX
    }

    pub fn degree(&self, node: usize) -> usize {
        if self.contains(node) {
            self.neighbours[self.slots[node]].len()
        } else {
            0
        }
    }

    pub fn distance(&self, from: usize, to: usize) -> usize {
        self.distances[self.slots[from]][self.slots[to]]
    }

    pub fn twins_ok(&self, path: &[usize], temperature: Temperature) -> bool {
        // Full signature validation is reserved for shortlisted paths.
        let mut neighbours = self.neighbours.clone();
        let mut added = HashMap::new();
        for &node in path.iter().filter(|&&node| !self.contains(node)) {
            added.insert(node, neighbours.len());
            neighbours.push(Vec::new());
        }
        let slot = |node| {
            if self.contains(node) {
                self.slots[node]
            } else {
                added[&node]
            }
        };
        for pair in path.windows(2) {
            neighbours[slot(pair[0])].push(pair[1]);
            neighbours[slot(pair[1])].push(pair[0]);
        }
        let mut signatures = HashMap::<_, usize>::new();
        for mut others in neighbours {
            others.sort_unstable();
            let count = signatures.entry(others).or_default();
            *count += 1;
            if *count > temperature.blend(2.0, 5.0).round() as usize {
                return false;
            }
        }
        true
    }

    pub fn quality(&self, sample: &Sample, temperature: Temperature, ring_size: usize) -> f32 {
        let degrees: Vec<_> = self.neighbours.iter().map(Vec::len).collect();
        let average = degrees.iter().sum::<usize>() as f32 / degrees.len() as f32;
        let variance = degrees
            .iter()
            .map(|&degree| (degree as f32 - average).powi(2))
            .sum::<f32>()
            / degrees.len() as f32;
        let ring_error = sample
            .cycles
            .iter()
            .map(|&size| (size as f32 - ring_size as f32).abs())
            .sum::<f32>()
            / sample.cycles.len() as f32;
        entropy(self.kinds.values().copied().collect()) * 5.0
            + entropy(self.relations.values().copied().collect()) * 4.0
            - variance * temperature.blend(3.0, -0.4)
            - ring_error * temperature.blend(1.8, 0.1)
            - self.leaves as f32 * temperature.blend(5.0, 0.2)
    }
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
