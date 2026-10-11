use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EntryKind {
    Block,
    Item,
}

impl EntryKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Block => "方块",
            Self::Item => "物品",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum MatchQuality {
    Exact,
    Partial,
    Wrong,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Attribute {
    pub label: String,
    pub values: Vec<String>,
}

impl Attribute {
    pub fn compare(&self, other: &Self) -> MatchQuality {
        let ours: BTreeSet<_> = self.values.iter().collect();
        let theirs: BTreeSet<_> = other.values.iter().collect();
        if ours == theirs {
            MatchQuality::Exact
        } else if !ours.is_disjoint(&theirs) {
            MatchQuality::Partial
        } else {
            MatchQuality::Wrong
        }
    }

    pub fn text(&self) -> String {
        self.values.join("、")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: EntryKind,
    pub attributes: Vec<Attribute>,
}
