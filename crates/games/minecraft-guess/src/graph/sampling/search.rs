use super::index::GraphIndex;
use super::mesh::Sample;
use super::temperature::Temperature;
use fastrand::Rng;
use std::collections::{HashMap, VecDeque};

pub(super) struct Search<'a> {
    pub index: &'a GraphIndex,
    pub random: Rng,
    pub temperature: Temperature,
    pub tag_limit: usize,
    pub remaining: usize,
    pub ring_size: usize,
    pub density: f32,
}

impl Search<'_> {
    pub fn neighbours(&mut self, node: usize, limit: usize, core_only: bool) -> Vec<usize> {
        self.index
            .neighbours(node, limit, core_only, &mut self.random)
    }

    pub fn spend(&mut self) -> bool {
        if self.remaining == 0 {
            return false;
        }
        self.remaining -= 1;
        true
    }

    pub fn cycle(&mut self) -> Option<Sample> {
        self.ring_size = self.temperature.ring_size(&mut self.random);
        self.density = self.temperature.density(&mut self.random);
        let roots: Vec<_> = (0..24)
            .map(|_| self.index.starts[self.random.usize(..self.index.starts.len())])
            .collect();
        let root = self.temperature.choose(
            roots,
            |&node| {
                (self.index.adjacent[node].len() as f32).ln() * self.temperature.blend(-1.8, 0.8)
            },
            &mut self.random,
        )?;
        let neighbours = self.neighbours(root, 10, true);
        let mut cycles = Vec::new();
        for pair in neighbours.windows(2) {
            let mut queue = VecDeque::from([vec![pair[0]]]);
            for _ in 0..100 {
                if !self.spend() {
                    break;
                }
                let path = if self.random.f32() < self.temperature.blend(0.0, 0.9) {
                    queue.pop_back()
                } else {
                    queue.pop_front()
                };
                let Some(path) = path else { break };
                let last = *path.last().expect("nonempty path");
                for next in self.neighbours(last, 12, true) {
                    if next == pair[1] {
                        let mut cycle = vec![root];
                        cycle.extend(&path);
                        cycle.push(next);
                        let tags = cycle
                            .iter()
                            .filter(|&&node| self.index.data.nodes[node].entry.is_none())
                            .count();
                        if tags <= self.tag_limit {
                            cycles.push(cycle);
                        }
                    } else if path.len() < self.ring_size.saturating_sub(2).max(2)
                        && next != root
                        && !path.contains(&next)
                    {
                        let mut branch = path.clone();
                        branch.push(next);
                        queue.push_back(branch);
                    }
                }
                if cycles.len() >= 24 {
                    break;
                }
            }
            if cycles.len() >= 24 {
                break;
            }
        }
        let mut cycle = self.temperature.choose(
            cycles,
            |cycle| -(cycle.len() as f32 - self.ring_size as f32).abs() * 2.0,
            &mut self.random,
        )?;
        let length = cycle.len();
        cycle.push(root);
        let edges = self
            .index
            .edges(&cycle, &HashMap::new(), self.temperature, &mut self.random);
        let mut sample = Sample {
            nodes: Vec::new(),
            edges: Vec::new(),
            cycles: Vec::new(),
        };
        sample.append(&cycle, edges, Some(length));
        Some(sample)
    }
}
