use super::GraphRelation;
use crate::{Entry, EntryKind};
use serde::{Deserialize, Serialize};

/// Slot numbers carry no native answer IDs until the node is revealed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphNodeView {
    pub number: u16,
    pub kind: Option<EntryKind>,
    pub character_count: Option<u16>,
    pub entry: Option<Entry>,
    pub tag_name: Option<String>,
}

impl GraphNodeView {
    pub fn revealed(&self) -> bool {
        self.entry.is_some() || self.tag_name.is_some()
    }

    pub fn label(&self) -> &str {
        self.entry
            .as_ref()
            .map(|entry| entry.name.as_str())
            .or(self.tag_name.as_deref())
            .unwrap_or_else(|| self.kind.map_or("标签", EntryKind::label))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphEdgeView {
    pub source: u16,
    pub target: u16,
    pub relation: GraphRelation,
    pub active: bool,
    /// None conceals orientation, Some(false) denotes a bidirectional relation.
    pub directed: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphView {
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdgeView>,
    pub selected: u16,
}

impl GraphView {
    pub fn selected_node(&self) -> &GraphNodeView {
        &self.nodes[usize::from(self.selected - 1)]
    }
}
