use super::data::GraphData;
use super::{GraphEdgeView, GraphNodeView, GraphView};
use crate::EntryKind;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

struct KnowledgeIndex {
    data: &'static GraphData,
    entries: HashMap<EntryKind, HashMap<&'static str, usize>>,
    tags: HashMap<&'static str, usize>,
    incident: Vec<Vec<usize>>,
}

impl KnowledgeIndex {
    fn bundled() -> &'static Self {
        static INDEX: OnceLock<KnowledgeIndex> = OnceLock::new();
        INDEX.get_or_init(|| {
            let data = GraphData::bundled();
            let mut entries: HashMap<_, HashMap<_, _>> = HashMap::new();
            let mut tags = HashMap::new();
            for (index, node) in data.nodes.iter().enumerate() {
                if let Some(entry) = node.entry {
                    entries
                        .entry(entry.kind)
                        .or_default()
                        .insert(entry.id.as_str(), index);
                } else {
                    tags.insert(node.name.as_str(), index);
                }
            }
            let mut incident = vec![Vec::new(); data.nodes.len()];
            for (index, edge) in data.edges.iter().enumerate() {
                incident[edge.from].push(index);
                if edge.to != edge.from {
                    incident[edge.to].push(index);
                }
            }
            Self {
                data,
                entries,
                tags,
                incident,
            }
        })
    }

    fn revealed_node(&self, node: &GraphNodeView) -> Option<usize> {
        if let Some(entry) = &node.entry {
            self.entries
                .get(&entry.kind)?
                .get(entry.id.as_str())
                .copied()
        } else {
            self.tags.get(node.tag_name.as_deref()?).copied()
        }
    }

    fn candidates(
        &self,
        known: &GraphNodeView,
        hint: &GraphEdgeView,
        target: u16,
        kind: EntryKind,
    ) -> HashSet<String> {
        let Some(known) = self.revealed_node(known) else {
            return HashSet::new();
        };
        self.incident[known]
            .iter()
            .filter_map(|&index| {
                let edge = &self.data.edges[index];
                if edge.kind.coarse() != hint.relation.coarse() {
                    return None;
                }
                let direction_matches = match hint.directed {
                    None => true,
                    Some(false) => !edge.directed,
                    Some(true) => {
                        edge.directed
                            && if hint.source == target {
                                edge.to == known
                            } else {
                                edge.from == known
                            }
                    }
                };
                if !direction_matches {
                    return None;
                }
                let other = if edge.from == known {
                    edge.to
                } else {
                    edge.from
                };
                let entry = self.data.nodes[other].entry?;
                (entry.kind == kind).then(|| entry.id.clone())
            })
            .collect()
    }
}

impl GraphView {
    /// Only unlocked neighbours constrain the target; hidden neighbours add no evidence.
    pub(crate) fn candidates_from_revealed(&self, target: u16) -> Option<HashSet<String>> {
        let kind = self.nodes.iter().find(|node| node.number == target)?.kind?;
        let mut candidates: Option<HashSet<String>> = None;
        for edge in &self.edges {
            let other = if edge.source == target {
                edge.target
            } else if edge.target == target {
                edge.source
            } else {
                continue;
            };
            let Some(known) = self
                .nodes
                .iter()
                .find(|node| node.number == other && node.revealed())
            else {
                continue;
            };
            let matching = KnowledgeIndex::bundled().candidates(known, edge, target, kind);
            if let Some(candidates) = &mut candidates {
                candidates.retain(|entry| matching.contains(entry));
            } else {
                candidates = Some(matching);
            }
        }
        candidates
    }
}
