use super::super::{McGuessEntry, McGuessEntryKind};
use super::{CatalogueReader, DefaultReader, EnchantmentReader, ItemReader, LandscapeReader};

pub(in super::super) fn read_bundled() -> Vec<McGuessEntry> {
    [
        DefaultReader.read(
            McGuessEntryKind::Block,
            include_str!("../../../../../../assets/data/minecraft/blocks.csv"),
        ),
        ItemReader.read(
            McGuessEntryKind::Item,
            include_str!("../../../../../../assets/data/minecraft/items.csv"),
        ),
        DefaultReader.read(
            McGuessEntryKind::Mob,
            include_str!("../../../../../../assets/data/minecraft/mobs.csv"),
        ),
        EnchantmentReader.read(
            McGuessEntryKind::Enchantment,
            include_str!("../../../../../../assets/data/minecraft/enchantments.csv"),
        ),
        DefaultReader.read(
            McGuessEntryKind::Effect,
            include_str!("../../../../../../assets/data/minecraft/effects.csv"),
        ),
        DefaultReader.read(
            McGuessEntryKind::VillagerProfession,
            include_str!("../../../../../../assets/data/minecraft/villager-professions.csv"),
        ),
        LandscapeReader.read(
            McGuessEntryKind::Landscape,
            include_str!("../../../../../../assets/data/minecraft/landscapes.csv"),
        ),
        DefaultReader.read(
            McGuessEntryKind::Biome,
            include_str!("../../../../../../assets/data/minecraft/biomes.csv"),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}
