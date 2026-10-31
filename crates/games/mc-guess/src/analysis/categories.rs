use crate::McGuessEntryKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessCategoryCount {
    pub games: u32,
    pub wins: u32,
}

/// Shared category counters for completed questions and graph nodes.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct McGuessCategoryStatistics {
    pub block: McGuessCategoryCount,
    pub item: McGuessCategoryCount,
    pub mob: McGuessCategoryCount,
    pub enchantment: McGuessCategoryCount,
    pub effect: McGuessCategoryCount,
    pub profession: McGuessCategoryCount,
    pub landscape: McGuessCategoryCount,
    pub biome: McGuessCategoryCount,
}

impl McGuessCategoryStatistics {
    pub fn get(&self, kind: McGuessEntryKind) -> &McGuessCategoryCount {
        match kind {
            McGuessEntryKind::Block => &self.block,
            McGuessEntryKind::Item => &self.item,
            McGuessEntryKind::Mob => &self.mob,
            McGuessEntryKind::Enchantment => &self.enchantment,
            McGuessEntryKind::Effect => &self.effect,
            McGuessEntryKind::VillagerProfession => &self.profession,
            McGuessEntryKind::Landscape => &self.landscape,
            McGuessEntryKind::Biome => &self.biome,
        }
    }

    pub(crate) fn record(&mut self, kind: McGuessEntryKind, won: bool) {
        let count = match kind {
            McGuessEntryKind::Block => &mut self.block,
            McGuessEntryKind::Item => &mut self.item,
            McGuessEntryKind::Mob => &mut self.mob,
            McGuessEntryKind::Enchantment => &mut self.enchantment,
            McGuessEntryKind::Effect => &mut self.effect,
            McGuessEntryKind::VillagerProfession => &mut self.profession,
            McGuessEntryKind::Landscape => &mut self.landscape,
            McGuessEntryKind::Biome => &mut self.biome,
        };
        count.games = count.games.saturating_add(1);
        count.wins = count.wins.saturating_add(u32::from(won));
    }

    pub fn merge(&mut self, other: Self) {
        for (total, count) in [
            (&mut self.block, other.block),
            (&mut self.item, other.item),
            (&mut self.mob, other.mob),
            (&mut self.enchantment, other.enchantment),
            (&mut self.effect, other.effect),
            (&mut self.profession, other.profession),
            (&mut self.landscape, other.landscape),
            (&mut self.biome, other.biome),
        ] {
            total.games = total.games.saturating_add(count.games);
            total.wins = total.wins.saturating_add(count.wins);
        }
    }

    pub fn total(&self) -> McGuessCategoryCount {
        [
            self.block,
            self.item,
            self.mob,
            self.enchantment,
            self.effect,
            self.profession,
            self.landscape,
            self.biome,
        ]
        .into_iter()
        .fold(McGuessCategoryCount::default(), |mut total, count| {
            total.games = total.games.saturating_add(count.games);
            total.wins = total.wins.saturating_add(count.wins);
            total
        })
    }
}
