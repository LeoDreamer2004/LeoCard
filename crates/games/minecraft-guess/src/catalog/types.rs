use super::BiomeRanges;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum EntryKind {
    Block,
    Item,
    Mob,
    Enchantment,
    Effect,
    VillagerProfession,
    Landscape,
    Biome,
}

impl EntryKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Block => "方块",
            Self::Item => "物品",
            Self::Mob => "生物",
            Self::Enchantment => "附魔",
            Self::Effect => "状态效果",
            Self::VillagerProfession => "村民职业",
            Self::Landscape => "世界景观",
            Self::Biome => "生物群系",
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
        self.display_values().join("、")
    }

    pub fn display_values(&self) -> Vec<String> {
        match self.label.as_str() {
            "生成生物群系" => BiomeRanges::bundled().describe(&self.values),
            "生成群系" => BiomeRanges::bundled().describe_by_dimension(&self.values),
            _ => self.values.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub kind: EntryKind,
    pub attributes: Vec<Attribute>,
}
