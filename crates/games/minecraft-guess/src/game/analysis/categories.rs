use crate::EntryKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessCategoryCount {
    pub games: u32,
    pub wins: u32,
}

/// Shared category counters for completed questions and graph nodes.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessCategoryStatistics {
    pub block: GuessCategoryCount,
    pub item: GuessCategoryCount,
    pub mob: GuessCategoryCount,
    pub enchantment: GuessCategoryCount,
    pub effect: GuessCategoryCount,
    pub profession: GuessCategoryCount,
    pub landscape: GuessCategoryCount,
    pub biome: GuessCategoryCount,
}

impl GuessCategoryStatistics {
    pub fn get(&self, kind: EntryKind) -> &GuessCategoryCount {
        match kind {
            EntryKind::Block => &self.block,
            EntryKind::Item => &self.item,
            EntryKind::Mob => &self.mob,
            EntryKind::Enchantment => &self.enchantment,
            EntryKind::Effect => &self.effect,
            EntryKind::VillagerProfession => &self.profession,
            EntryKind::Landscape => &self.landscape,
            EntryKind::Biome => &self.biome,
        }
    }

    pub(crate) fn record(&mut self, kind: EntryKind, won: bool) {
        let count = match kind {
            EntryKind::Block => &mut self.block,
            EntryKind::Item => &mut self.item,
            EntryKind::Mob => &mut self.mob,
            EntryKind::Enchantment => &mut self.enchantment,
            EntryKind::Effect => &mut self.effect,
            EntryKind::VillagerProfession => &mut self.profession,
            EntryKind::Landscape => &mut self.landscape,
            EntryKind::Biome => &mut self.biome,
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

    pub fn total(&self) -> GuessCategoryCount {
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
        .fold(GuessCategoryCount::default(), |mut total, count| {
            total.games = total.games.saturating_add(count.games);
            total.wins = total.wins.saturating_add(count.wins);
            total
        })
    }
}
