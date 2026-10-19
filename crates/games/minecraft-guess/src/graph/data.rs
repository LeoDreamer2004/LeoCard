use super::GraphRelation;
use crate::{Catalogue, Entry, EntryKind};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

pub(super) struct GraphNode {
    pub entry: Option<&'static Entry>,
    pub name: String,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct GraphEdge {
    pub from: usize,
    pub to: usize,
    pub kind: GraphRelation,
    pub directed: bool,
}

pub(super) struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
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
    kind: GraphRelation,
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
            let nodes = raw
                .nodes
                .into_iter()
                .map(|node| {
                    let kind = match node.kind.as_str() {
                        "block" => Some(EntryKind::Block),
                        "item" => Some(EntryKind::Item),
                        "mob" => Some(EntryKind::Mob),
                        "enchantment" => Some(EntryKind::Enchantment),
                        "effect" => Some(EntryKind::Effect),
                        "villager_profession" => Some(EntryKind::VillagerProfession),
                        "landscape" => Some(EntryKind::Landscape),
                        "biome" => Some(EntryKind::Biome),
                        "tag" => None,
                        _ => panic!("non-catalogue node in published graph"),
                    };
                    let entry = kind.map(|kind| {
                        Catalogue::bundled()
                            .find(kind, node.id.split_once(':').expect("typed graph ID").1)
                            .expect("graph node belongs to catalogue")
                    });
                    GraphNode {
                        entry,
                        name: node.name,
                    }
                })
                .collect();
            let edges = raw
                .edges
                .into_iter()
                .map(|raw| GraphEdge {
                    from: indices[&raw.source],
                    to: indices[&raw.target],
                    kind: raw.kind,
                    directed: raw.directed,
                })
                .collect();
            Self { nodes, edges }
        })
    }
}
