use super::GraphRelation;
use crate::{Entry, EntryKind, GuessRecord};
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
pub struct GraphGuessRecord {
    pub node: u16,
    pub guess: GuessRecord,
    /// Successful streak length, starting at one; zero for an incorrect guess.
    pub combo: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphCombo {
    pub count: u16,
    /// Authoritative playable targets; neighbours take priority over all other nodes.
    pub targets: Vec<u16>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GraphView {
    pub nodes: Vec<GraphNodeView>,
    pub edges: Vec<GraphEdgeView>,
    pub selected: u16,
    pub combo: Option<GraphCombo>,
    /// Public guesses in acceptance order; unrevealed answers are never included.
    pub guesses: Vec<GraphGuessRecord>,
}

impl GraphView {
    pub fn can_guess_node(&self, number: u16) -> bool {
        number
            .checked_sub(1)
            .and_then(|index| self.nodes.get(usize::from(index)))
            .is_some_and(|node| node.kind.is_some() && !node.revealed())
            && self
                .combo
                .as_ref()
                .is_none_or(|combo| combo.targets.contains(&number))
    }

    pub fn number_label(&self, number: u16) -> String {
        let index = usize::from(number - 1);
        if self.nodes[index].kind.is_some() {
            return number.to_string();
        }
        let mut ordinal = self.nodes[..=index]
            .iter()
            .filter(|node| node.kind.is_none())
            .count();
        let mut letters = Vec::new();
        while ordinal > 0 {
            ordinal -= 1;
            letters.push(char::from(b'A' + (ordinal % 26) as u8));
            ordinal /= 26;
        }
        format!("#{}", letters.into_iter().rev().collect::<String>())
    }

    pub fn selected_node(&self) -> &GraphNodeView {
        &self.nodes[usize::from(self.selected - 1)]
    }
}
