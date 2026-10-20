use super::super::GraphRelation;
use super::super::data::{GraphData, GraphEdge};
use super::temperature::Temperature;
use fastrand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::OnceLock;

pub(super) struct GraphIndex {
    pub data: &'static GraphData,
    pub adjacent: Vec<Vec<usize>>,
    pub starts: Vec<usize>,
    core: Vec<Vec<usize>>,
    pairs: HashMap<(usize, usize), Vec<GraphEdge>>,
}

impl GraphIndex {
    pub fn bundled(tags: bool) -> &'static Self {
        static WITH_TAGS: OnceLock<GraphIndex> = OnceLock::new();
        static WITHOUT_TAGS: OnceLock<GraphIndex> = OnceLock::new();
        let cache = if tags { &WITH_TAGS } else { &WITHOUT_TAGS };
        cache.get_or_init(|| Self::new(GraphData::bundled(), tags))
    }

    fn new(data: &'static GraphData, include_tags: bool) -> Self {
        let mut adjacent = vec![Vec::new(); data.nodes.len()];
        let mut pairs: HashMap<_, Vec<_>> = HashMap::new();
        for edge in &data.edges {
            let tags = [edge.from, edge.to].map(|i| data.nodes[i].entry.is_none());
            if edge.from == edge.to || tags == [true, true] || !include_tags && tags.contains(&true)
            {
                continue;
            }
            let key = (edge.from.min(edge.to), edge.from.max(edge.to));
            let choices = pairs.entry(key).or_default();
            if choices.is_empty() {
                adjacent[edge.from].push(edge.to);
                adjacent[edge.to].push(edge.from);
            }
            choices.push(*edge);
        }
        let mut degree: Vec<_> = adjacent.iter().map(Vec::len).collect();
        let mut queue: VecDeque<_> = (0..degree.len()).filter(|&i| degree[i] < 2).collect();
        let mut core = vec![true; degree.len()];
        while let Some(node) = queue.pop_front() {
            if !core[node] {
                continue;
            }
            core[node] = false;
            for &other in &adjacent[node] {
                if core[other] {
                    degree[other] -= 1;
                    if degree[other] == 1 {
                        queue.push_back(other);
                    }
                }
            }
        }
        let mut core_adjacent = adjacent.clone();
        for (node, neighbours) in core_adjacent.iter_mut().enumerate() {
            if core[node] {
                neighbours.retain(|&other| core[other]);
            } else {
                neighbours.clear();
            }
        }
        let starts = core_adjacent
            .iter()
            .enumerate()
            .filter_map(|(node, neighbours)| {
                (neighbours.len() >= 2 && data.nodes[node].entry.is_some()).then_some(node)
            })
            .collect();
        Self {
            data,
            adjacent,
            starts,
            core: core_adjacent,
            pairs,
        }
    }

    pub fn neighbours(
        &self,
        node: usize,
        limit: usize,
        core_only: bool,
        random: &mut Rng,
    ) -> Vec<usize> {
        let adjacent = if core_only {
            &self.core[node]
        } else {
            &self.adjacent[node]
        };
        if adjacent.len() <= limit {
            let mut result = adjacent.clone();
            random.shuffle(&mut result);
            return result;
        }
        // Sample a small prefix without copying/shuffling a high-degree hub's whole list.
        let mut result = Vec::with_capacity(limit);
        while result.len() < limit {
            let other = adjacent[random.usize(..adjacent.len())];
            if !result.contains(&other) {
                result.push(other);
            }
        }
        result
    }

    pub fn edges(
        &self,
        path: &[usize],
        counts: &HashMap<GraphRelation, usize>,
        temperature: Temperature,
        random: &mut Rng,
    ) -> Vec<GraphEdge> {
        let mut edges: Vec<GraphEdge> = Vec::with_capacity(path.len() - 1);
        for pair in path.windows(2) {
            let key = (pair[0].min(pair[1]), pair[0].max(pair[1]));
            let edge = temperature
                .choose(
                    self.pairs[&key].iter(),
                    |edge| {
                        -((counts.get(&edge.kind).copied().unwrap_or(0)
                            + edges.iter().filter(|other| other.kind == edge.kind).count())
                            as f32)
                    },
                    random,
                )
                .expect("real edge");
            edges.push(*edge);
        }
        edges
    }
}
