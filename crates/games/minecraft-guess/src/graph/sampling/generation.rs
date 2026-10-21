use super::super::data::GraphData;
use super::index::GraphIndex;
use super::mesh::{Sample, Topology};
use super::search::Search;
use super::temperature::Temperature;
use crate::GraphRules;
use fastrand::Rng;

impl Sample {
    pub(in super::super) fn generate(rules: GraphRules, seed: u64) -> Option<Self> {
        let size = usize::from(rules.nodes);
        if !GraphRules::NODE_OPTIONS.contains(&rules.nodes) || rules.temperature > 10 {
            return None;
        }
        if size == 2 {
            return Self::pair(seed);
        }
        let index = GraphIndex::bundled(rules.include_tags);
        if index.starts.is_empty() {
            return None;
        }
        let mut search = Search {
            index,
            random: Rng::with_seed(seed),
            temperature: Temperature::new(rules.temperature),
            tag_limit: if rules.include_tags {
                (size / 8).max(2)
            } else {
                0
            },
            remaining: 120_000,
            ring_size: 4,
            density: 1.6,
        };
        let mut complete = Vec::new();
        for _ in 0..16 {
            let Some(mut sample) = search.cycle() else {
                continue;
            };
            let mut topology = Topology::new(index, &sample);
            while sample.nodes.len() - topology.tags < size {
                let Some(candidate) = search.grow(&sample, &topology, size) else {
                    break;
                };
                sample.append(&candidate.path, candidate.edges, candidate.cycle);
                topology = Topology::new(index, &sample);
            }
            if sample.nodes.len() - topology.tags == size {
                search.chords(&mut sample, size);
                topology = Topology::new(index, &sample);
                let quality = topology.quality(&sample, search.temperature, search.ring_size)
                    + index.rarity_score(sample.nodes.iter().copied(), search.temperature)
                    + topology.diversity.score();
                complete.push((sample, quality));
            }
            if complete.len() >= 4 || search.remaining == 0 {
                break;
            }
        }
        search
            .temperature
            .choose(complete, |(_, score)| *score, &mut search.random)
            .map(|(mut sample, _)| {
                // Keep playable numbers consecutive; tags retain unique slots after them.
                sample
                    .nodes
                    .sort_by_key(|&node| index.data.nodes[node].entry.is_none());
                sample
            })
    }

    /// The developer preview is a real relation between two playable entries,
    /// without tags or the cycle search used by full puzzles.
    fn pair(seed: u64) -> Option<Self> {
        let data = GraphData::bundled();
        let edge = Rng::with_seed(seed)
            .choose_multiple(
                data.edges.iter().filter(|edge| {
                    edge.from != edge.to
                        && data.nodes[edge.from].entry.is_some()
                        && data.nodes[edge.to].entry.is_some()
                }),
                1,
            )
            .pop()
            .copied()?;
        Some(Self {
            nodes: vec![edge.from, edge.to],
            edges: vec![edge],
            cycles: Vec::new(),
        })
    }
}
