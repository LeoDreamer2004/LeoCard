use super::super::data::GraphEdge;
use super::index::GraphIndex;
use super::mesh::{Sample, Topology};
use fastrand::Rng;
use std::collections::{HashMap, HashSet, VecDeque};

pub(super) struct Search<'a> {
    pub index: &'a GraphIndex,
    pub random: Rng,
    pub tag_limit: usize,
    pub remaining: usize,
}

pub(super) struct Candidate {
    pub path: Vec<usize>,
    pub edges: Vec<GraphEdge>,
    score: f32,
}

impl Search<'_> {
    fn neighbours(&mut self, node: usize, limit: usize) -> Vec<usize> {
        self.index.neighbours(node, limit, &mut self.random)
    }

    pub fn cycle(&mut self) -> Option<Sample> {
        let root = self.index.starts[self.random.usize(..self.index.starts.len())];
        let neighbours = self.neighbours(root, 10);
        for pair in neighbours.windows(2) {
            let mut queue = VecDeque::from([vec![pair[0]]]);
            let mut visited = vec![false; self.index.data.nodes.len()];
            visited[root] = true;
            visited[pair[0]] = true;
            for _ in 0..160 {
                let Some(path) = queue.pop_front() else { break };
                let last = *path.last().expect("nonempty path");
                for next in self.neighbours(last, 18) {
                    if next == pair[1] {
                        let mut cycle = vec![root];
                        cycle.extend(&path);
                        cycle.push(next);
                        if cycle
                            .iter()
                            .filter(|&&node| self.index.data.nodes[node].entry.is_none())
                            .count()
                            > self.tag_limit
                        {
                            continue;
                        }
                        let mut sample = Sample {
                            nodes: Vec::new(),
                            edges: Vec::new(),
                        };
                        cycle.push(root);
                        let edges = self.index.edges(&cycle, &HashMap::new());
                        sample.append(&cycle, edges);
                        return Some(sample);
                    }
                    if path.len() < 3 && !visited[next] {
                        visited[next] = true;
                        let mut branch = path.clone();
                        branch.push(next);
                        queue.push_back(branch);
                    }
                }
            }
        }
        None
    }

    pub fn ear(&mut self, sample: &Sample, topology: &Topology, size: usize) -> Option<Candidate> {
        let mut ends: Vec<_> = sample
            .nodes
            .iter()
            .copied()
            .filter(|&node| topology.degree(node) < 4)
            .collect();
        self.random.shuffle(&mut ends);
        ends.truncate(12);
        let mut candidates = Vec::new();
        let mut seen = HashSet::new();
        for start in ends {
            let mut queue: VecDeque<_> = self
                .neighbours(start, 14)
                .into_iter()
                .filter(|&node| !topology.contains(node))
                .map(|node| vec![start, node])
                .collect();
            for _ in 0..70 {
                let Some(path) = queue.pop_front() else { break };
                if self.remaining == 0 {
                    break;
                }
                self.remaining -= 1;
                let adjacent = self.neighbours(*path.last().expect("path"), 18);
                for next in adjacent
                    .iter()
                    .copied()
                    .filter(|&node| {
                        node != start && topology.contains(node) && topology.degree(node) < 4
                    })
                    .take(3)
                {
                    if sample.edges.len() + path.len() > size * 145 / 100 {
                        continue;
                    }
                    let mut candidate = path.clone();
                    candidate.push(next);
                    let key: Vec<_> = if start < next {
                        candidate.clone()
                    } else {
                        candidate.iter().rev().copied().collect()
                    };
                    if !seen.insert(key) {
                        continue;
                    }
                    if let Some(candidate) = self.candidate(candidate, topology) {
                        candidates.push(candidate);
                    }
                }
                if path.len() - 1 < (size - sample.nodes.len()).min(3) {
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
        candidates
            .into_iter()
            .take(32)
            .find(|candidate| topology.twins_ok(&candidate.path))
    }

    fn candidate(&mut self, path: Vec<usize>, topology: &Topology) -> Option<Candidate> {
        let mut new_kinds = HashSet::new();
        let mut tags = topology.tags;
        let mut balance = 0.0;
        for &node in &path[1..path.len() - 1] {
            let kind = self.index.data.nodes[node].entry.map(|entry| entry.kind);
            tags += usize::from(kind.is_none());
            if !topology.kinds.contains_key(&kind) {
                new_kinds.insert(kind);
            }
            balance += 1.0 / (1.0 + topology.kinds.get(&kind).copied().unwrap_or(0) as f32 / 3.0);
        }
        if tags > self.tag_limit {
            return None;
        }
        let edges = self.index.edges(&path, &topology.relations);
        let new_relations: HashSet<_> = edges
            .iter()
            .filter_map(|edge| (!topology.relations.contains_key(&edge.kind)).then_some(edge.kind))
            .collect();
        let cost =
            (topology.degree(path[0]) + topology.degree(*path.last().unwrap())) as f32 * 0.35;
        let score = new_kinds.len() as f32 * 3.0
            + new_relations.len() as f32 * 2.5
            + balance
            + (path.len() - 1) as f32 * 0.5
            - cost
            + self.random.f32() * 0.6;
        Some(Candidate { path, edges, score })
    }
}
