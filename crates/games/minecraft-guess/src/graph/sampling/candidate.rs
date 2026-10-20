use super::super::data::GraphEdge;
use super::mesh::Topology;
use super::search::Search;
use std::collections::HashSet;

pub(super) struct Candidate {
    pub path: Vec<usize>,
    pub edges: Vec<GraphEdge>,
    pub cycle: Option<usize>,
    pub score: f32,
}

impl Search<'_> {
    pub fn candidate(&mut self, path: Vec<usize>, topology: &Topology) -> Option<Candidate> {
        let mut new_kinds = HashSet::new();
        let mut tags = topology.tags;
        let mut balance = 0.0;
        for &node in path.iter().filter(|&&node| !topology.contains(node)) {
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
        let edges = self.index.edges(
            &path,
            &topology.relations,
            self.temperature,
            &mut self.random,
        );
        let new_relations: HashSet<_> = edges
            .iter()
            .filter_map(|edge| (!topology.relations.contains_key(&edge.kind)).then_some(edge.kind))
            .collect();
        let start = path[0];
        let end = *path.last().expect("candidate path");
        let closed = topology.contains(end);
        let cycle = closed.then(|| topology.distance(start, end) + path.len() - 1);
        let load = (topology.degree(start) + topology.degree(end)) as f32;
        let ring_error = cycle.map_or(0.0, |size| (size as f32 - self.ring_size as f32).abs());
        let score = new_kinds.len() as f32 * 3.0 + new_relations.len() as f32 * 2.5 + balance
            - load * self.temperature.blend(1.2, -0.4)
            - ring_error * self.temperature.blend(1.8, 0.25)
            - (path.len() - 2) as f32 * self.temperature.blend(0.6, -0.15);
        Some(Candidate {
            path,
            edges,
            cycle,
            score,
        })
    }
}
