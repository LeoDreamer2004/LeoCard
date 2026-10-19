use super::super::{Entry, EntryKind};
use super::{CatalogueReader, DefaultReader, EnchantmentReader, ItemReader, LandscapeReader};

pub(in super::super) fn read_bundled() -> Vec<Entry> {
    [
        DefaultReader.read(
            EntryKind::Block,
            include_str!("../../../../../../assets/data/minecraft/blocks.csv"),
        ),
        ItemReader.read(
            EntryKind::Item,
            include_str!("../../../../../../assets/data/minecraft/items.csv"),
        ),
        DefaultReader.read(
            EntryKind::Mob,
            include_str!("../../../../../../assets/data/minecraft/mobs.csv"),
        ),
        EnchantmentReader.read(
            EntryKind::Enchantment,
            include_str!("../../../../../../assets/data/minecraft/enchantments.csv"),
        ),
        DefaultReader.read(
            EntryKind::Effect,
            include_str!("../../../../../../assets/data/minecraft/effects.csv"),
        ),
        DefaultReader.read(
            EntryKind::VillagerProfession,
            include_str!("../../../../../../assets/data/minecraft/villager-professions.csv"),
        ),
        LandscapeReader.read(
            EntryKind::Landscape,
            include_str!("../../../../../../assets/data/minecraft/landscapes.csv"),
        ),
        DefaultReader.read(
            EntryKind::Biome,
            include_str!("../../../../../../assets/data/minecraft/biomes.csv"),
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}
