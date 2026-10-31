use super::McGuessGraphRelation;
use crate::{McGuessCatalogue, McGuessEntry, McGuessEntryKind};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub(super) struct GraphNode {
    pub entry: Option<&'static McGuessEntry>,
    pub name: String,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub kind: McGuessGraphRelation,
    pub directed: bool,
}

pub(super) struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    degrees: Vec<usize>,
}

#[derive(Deserialize)]
struct RawNode {
    id: String,
    name: String,
    kind: String,
}

#[derive(Deserialize)]
struct RawEdge {
    source: String,
    target: String,
    kind: McGuessGraphRelation,
    #[serde(default = "default_directed")]
    directed: bool,
}

#[derive(Deserialize)]
struct RawData {
    nodes: Vec<RawNode>,
    edges: Vec<RawEdge>,
}

fn default_directed() -> bool {
    true
}

impl GraphData {
    pub fn bundled() -> &'static Self {
        static DATA: OnceLock<GraphData> = OnceLock::new();
        DATA.get_or_init(|| {
            let raw: RawData = serde_json::from_str(include_str!(
                "../../../../../assets/data/minecraft/knowledge-graph.json"
            ))
            .expect("validated Minecraft graph");
            let indices: HashMap<_, _> = raw
                .nodes
                .iter()
                .enumerate()
                .map(|(i, node)| (node.id.clone(), i))
                .collect();
            let nodes: Vec<_> = raw
                .nodes
                .into_iter()
                .map(|node| {
                    let kind = match node.kind.as_str() {
                        "block" => Some(McGuessEntryKind::Block),
                        "item" => Some(McGuessEntryKind::Item),
                        "mob" => Some(McGuessEntryKind::Mob),
                        "enchantment" => Some(McGuessEntryKind::Enchantment),
                        "effect" => Some(McGuessEntryKind::Effect),
                        "villager_profession" => Some(McGuessEntryKind::VillagerProfession),
                        "landscape" => Some(McGuessEntryKind::Landscape),
                        "biome" => Some(McGuessEntryKind::Biome),
                        "tag" => None,
                        _ => panic!("non-catalogue node in published graph"),
                    };
                    let entry = kind.map(|kind| {
                        McGuessCatalogue::bundled()
                            .find(kind, node.id.split_once(':').expect("typed graph ID").1)
                            .unwrap_or_else(|| {
                                panic!(
                                    "missing catalogue entry for graph node {} ({kind:?})",
                                    node.id
                                )
                            })
                    });
                    GraphNode {
                        entry,
                        name: node.name,
                    }
                })
                .collect();
            let edges: Vec<_> = raw
                .edges
                .into_iter()
                .map(|raw| GraphEdge {
                    from: indices[&raw.source],
                    to: indices[&raw.target],
                    kind: raw.kind,
                    directed: raw.directed,
                })
                .collect();
            let mut degrees = vec![0; nodes.len()];
            let mut pairs = HashSet::new();
            for edge in &edges {
                // Multiple relations or opposite arrows still describe one neighbour.
                let pair = (edge.from.min(edge.to), edge.from.max(edge.to));
                if edge.from != edge.to && pairs.insert(pair) {
                    degrees[edge.from] += 1;
                    degrees[edge.to] += 1;
                }
            }
            Self {
                nodes,
                edges,
                degrees,
            }
        })
    }

    pub fn degree(&self, node: usize) -> usize {
        self.degrees[node]
    }
}
