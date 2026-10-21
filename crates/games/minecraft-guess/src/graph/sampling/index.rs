use super::super::GraphRelation;
use super::super::data::{GraphData, GraphEdge};
use super::diversity::DiversityIndex;
use super::temperature::Temperature;
use fastrand::Rng;
use std::collections::{HashMap, VecDeque};
use std::sync::OnceLock;

pub(super) struct GraphIndex {
    pub data: &'static GraphData,
    pub adjacent: Vec<Vec<usize>>,
    pub starts: Vec<usize>,
    pub diversity: DiversityIndex,
    core: Vec<Vec<usize>>,
    source_degree_logs: Vec<f32>,
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
            diversity: DiversityIndex::new(data),
            source_degree_logs: (0..data.nodes.len())
                .map(|node| (data.degree(node).max(1) as f32).ln())
                .collect(),
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
        temperature: Temperature,
        random: &mut Rng,
    ) -> Vec<usize> {
        let adjacent = if core_only {
            &self.core[node]
        } else {
            &self.adjacent[node]
        };
        let cold = temperature.rarity_penalty(1.0) == 0.0;
        let pool_size = adjacent.len().min(if cold { limit } else { limit * 4 });
        let mut result = if adjacent.len() <= pool_size {
            let mut result = adjacent.clone();
            random.shuffle(&mut result);
            result
        } else {
            // Oversample a bounded pool without scanning a source hub's entire neighbourhood.
            let mut result = Vec::with_capacity(pool_size);
            while result.len() < pool_size {
                let other = adjacent[random.usize(..adjacent.len())];
                if !result.contains(&other) {
                    result.push(other);
                }
            }
            result
        };
        if !cold {
            result = temperature.prioritize(
                result,
                |&node| self.rarity_score([node], temperature),
                random,
            );
        }
        result.truncate(limit);
        result
    }

    pub fn rarity_score(
        &self,
        nodes: impl IntoIterator<Item = usize>,
        temperature: Temperature,
    ) -> f32 {
        let (sum, count, hubs) =
            nodes
                .into_iter()
                .fold((0.0, 0, 0.0), |(sum, count, hubs), node| {
                    let degree_log = self.source_degree_logs[node];
                    (
                        sum + degree_log,
                        count + 1,
                        hubs + temperature.hub_penalty(degree_log),
                    )
                });
        // A source hub pays its own cost; many rare neighbours cannot dilute it away.
        -temperature.rarity_penalty(sum / count.max(1) as f32) - hubs
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
