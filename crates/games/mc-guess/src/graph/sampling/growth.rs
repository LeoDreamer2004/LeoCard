use super::candidate::Candidate;
use super::mesh::{Sample, Topology};
use super::search::Search;
use std::collections::{HashSet, VecDeque};

impl Search<'_> {
    pub fn grow(&mut self, sample: &Sample, topology: &Topology, size: usize) -> Option<Candidate> {
        if self.temperature.branch(&mut self.random)
            && let Some(branch) = self.paths(sample, topology, size, false)
        {
            return Some(branch);
        }
        self.paths(sample, topology, size, true)
            .or_else(|| self.paths(sample, topology, size, false))
    }

    fn paths(
        &mut self,
        sample: &Sample,
        topology: &Topology,
        size: usize,
        closed: bool,
    ) -> Option<Candidate> {
        let leaf_limit = self.temperature.leaf_limit(size);
        if !closed && leaf_limit == 0 {
            return None;
        }
        let degree_limit = self.temperature.degree_limit();
        let mut ends: Vec<_> = sample
            .nodes
            .iter()
            .copied()
            .filter(|&node| {
                topology.degree(node) < degree_limit
                    && (closed || topology.leaves < leaf_limit || topology.degree(node) == 1)
            })
            .collect();
        self.random.shuffle(&mut ends);
        ends.truncate(12);
        let mut candidates = Vec::new();
        let mut seen = HashSet::new();
        for start in ends {
            let mut queue: VecDeque<_> = self
                .neighbours(start, 14, closed)
                .into_iter()
                .filter(|&node| !topology.contains(node))
                .map(|node| vec![start, node])
                .collect();
            for _ in 0..70 {
                let path = if self.random.f32() < self.temperature.blend(0.0, 0.75) {
                    queue.pop_back()
                } else {
                    queue.pop_front()
                };
                let Some(path) = path else { break };
                if !self.spend() {
                    break;
                }
                let tags = path[1..]
                    .iter()
                    .filter(|&&node| self.index.data.nodes[node].entry.is_none())
                    .count();
                let entries = path.len() - 1 - tags;
                let remaining = size - (sample.nodes.len() - topology.tags);
                if entries > remaining || topology.tags + tags > self.tag_limit {
                    continue;
                }
                let last = *path.last().expect("nonempty path");
                let adjacent = self.neighbours(last, 18, closed);
                if closed {
                    for next in adjacent
                        .iter()
                        .copied()
                        .filter(|&node| {
                            node != start
                                && topology.contains(node)
                                && topology.degree(node) < degree_limit
                        })
                        .take(4)
                    {
                        if sample.edges.len() + path.len()
                            > ((size + topology.tags + tags) as f32 * self.density) as usize
                        {
                            continue;
                        }
                        let mut candidate = path.clone();
                        candidate.push(next);
                        let key: Vec<_> = if start < next {
                            candidate.clone()
                        } else {
                            candidate.iter().rev().copied().collect()
                        };
                        if seen.insert(key)
                            && let Some(candidate) = self.candidate(candidate, topology)
                        {
                            candidates.push(candidate);
                        }
                    }
                } else if self.index.data.nodes[last].entry.is_some()
                    && let Some(candidate) = self.candidate(path.clone(), topology)
                {
                    candidates.push(candidate);
                }
                if path.len() - 1 < self.temperature.path_limit() {
                    for next in adjacent
                        .into_iter()
                        .filter(|&node| !topology.contains(node) && !path.contains(&node))
                        .take(6)
                    {
                        let mut branch = path.clone();
                        branch.push(next);
                        queue.push_back(branch);
                    }
                }
            }
        }
        candidates.sort_unstable_by(|a, b| b.score.total_cmp(&a.score));
        let shortlist: Vec<_> = candidates
            .into_iter()
            .take(48)
            .filter(|candidate| topology.twins_ok(&candidate.path, self.temperature))
            .collect();
        self.temperature
            .choose(shortlist, |candidate| candidate.score, &mut self.random)
    }

    pub fn chords(&mut self, sample: &mut Sample, size: usize) {
        let limit = (sample.nodes.len() as f32 * self.density) as usize;
        // Bounded enrichment, never an induced graph of every possible relation.
        for _ in 0..size / 2 {
            if sample.edges.len() >= limit || !self.spend() {
                break;
            }
            let topology = Topology::new(self.index, sample);
            let mut candidates = Vec::new();
            for (slot, &from) in sample.nodes.iter().enumerate() {
                if topology.degree(from) >= self.temperature.degree_limit() {
                    continue;
                }
                for &to in &sample.nodes[slot + 1..] {
                    if topology.degree(to) >= self.temperature.degree_limit()
                        || topology.distance(from, to) == 1
                        || !self.index.adjacent[from].contains(&to)
                    {
                        continue;
                    }
                    if let Some(candidate) = self.candidate(vec![from, to], &topology)
                        && topology.twins_ok(&candidate.path, self.temperature)
                    {
                        candidates.push(candidate);
                    }
                }
            }
            let Some(candidate) =
                self.temperature
                    .choose(candidates, |candidate| candidate.score, &mut self.random)
            else {
                break;
            };
            sample.append(&candidate.path, candidate.edges, candidate.cycle);
        }
    }
}
