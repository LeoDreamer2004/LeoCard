use super::McGuessGraphRelation;
use crate::{McGuessEntry, McGuessEntryKind, McGuessRecord};
use serde::{Deserialize, Serialize};

/// Slot numbers carry no native answer IDs until the node is revealed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphNodeView {
    pub number: u16,
    pub kind: Option<McGuessEntryKind>,
    pub character_count: Option<u16>,
    pub entry: Option<McGuessEntry>,
    pub tag_name: Option<String>,
}

impl McGuessGraphNodeView {
    pub fn revealed(&self) -> bool {
        self.entry.is_some() || self.tag_name.is_some()
    }

    pub fn label(&self) -> &str {
        self.entry
            .as_ref()
            .map(|entry| entry.name.as_str())
            .or(self.tag_name.as_deref())
            .unwrap_or_else(|| self.kind.map_or("标签", McGuessEntryKind::label))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphEdgeView {
    pub source: u16,
    pub target: u16,
    pub relation: McGuessGraphRelation,
    pub active: bool,
    /// None conceals orientation, Some(false) denotes a bidirectional relation.
    pub directed: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphGuessRecord {
    pub node: u16,
    pub guess: McGuessRecord,
    /// Successful streak length, starting at one; zero for an incorrect guess.
    pub combo: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphCombo {
    pub count: u16,
    /// Authoritative playable targets; neighbours take priority over all other nodes.
    pub targets: Vec<u16>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessGraphView {
    pub nodes: Vec<McGuessGraphNodeView>,
    pub edges: Vec<McGuessGraphEdgeView>,
    pub selected: u16,
    pub combo: Option<McGuessGraphCombo>,
    /// Public guesses in acceptance order; unrevealed answers are never included.
    pub guesses: Vec<McGuessGraphGuessRecord>,
}

impl McGuessGraphView {
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

    pub fn selected_node(&self) -> &McGuessGraphNodeView {
        &self.nodes[usize::from(self.selected - 1)]
    }
}
