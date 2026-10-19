use super::index::GraphIndex;
use super::mesh::{Sample, Topology};
use super::search::Search;
use fastrand::Rng;

impl Sample {
    pub(in super::super) fn generate(size: usize, tags: bool, seed: u64) -> Option<Self> {
        let index = GraphIndex::bundled(tags);
        if index.starts.is_empty() || size < 3 {
            return None;
        }
        let mut search = Search {
            index,
            random: Rng::with_seed(seed),
            tag_limit: if tags { (size / 8).max(2) } else { 0 },
            remaining: 120_000,
        };
        let mut best = None;
        let mut complete = 0;
        for _ in 0..16 {
            let Some(mut sample) = search.cycle() else {
                continue;
            };
            let mut topology = Topology::new(index, &sample);
            while sample.nodes.len() < size {
                let Some(candidate) = search.ear(&sample, &topology, size) else {
                    break;
                };
                sample.append(&candidate.path, candidate.edges);
                topology = Topology::new(index, &sample);
            }
            if sample.nodes.len() == size {
                complete += 1;
                let score = topology.quality(&sample);
                if best.as_ref().is_none_or(|(_, previous)| score > *previous) {
                    best = Some((sample, score));
                }
            }
            if complete >= 4 || search.remaining == 0 {
                break;
            }
        }
        best.map(|(sample, _)| sample)
    }
}
