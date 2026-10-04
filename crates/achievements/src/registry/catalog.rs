//! Shared immutable registry, lookup and public medal counts.
use super::table::combine;
use super::{
    AchievementCategory, AchievementDefinition, mahjong, personal, qigui523, shengji, texas, uno,
};
use leocard_protocol::AchievementCounts;
use std::collections::HashSet;

const COUNT: usize = mahjong::DEFINITIONS.len()
    + texas::DEFINITIONS.len()
    + shengji::DEFINITIONS.len()
    + personal::DEFINITIONS.len()
    + uno::DEFINITIONS.len()
    + qigui523::DEFINITIONS.len();
pub static ACHIEVEMENT_REGISTRY: &[AchievementDefinition] = &combine::<COUNT>(&[
    mahjong::DEFINITIONS,
    texas::DEFINITIONS,
    shengji::DEFINITIONS,
    personal::DEFINITIONS,
    uno::DEFINITIONS,
    qigui523::DEFINITIONS,
]);

pub fn achievement_by_id(id: &str) -> Option<&'static AchievementDefinition> {
    ACHIEVEMENT_REGISTRY
        .iter()
        .find(|definition| definition.id == id)
}

pub fn achievements_in(
    category: AchievementCategory,
) -> impl Iterator<Item = &'static AchievementDefinition> {
    ACHIEVEMENT_REGISTRY
        .iter()
        .filter(move |definition| definition.category == category)
}

pub fn achievement_counts<'a>(ids: impl IntoIterator<Item = &'a str>) -> AchievementCounts {
    let mut seen = HashSet::new();
    let mut counts = [0_u32; 3];
    for id in ids {
        if seen.insert(id)
            && let Some(definition) = achievement_by_id(id)
        {
            counts[definition.tier.medal_index()] += 1;
        }
    }
    AchievementCounts {
        gold: counts[0],
        silver: counts[1],
        bronze: counts[2],
    }
}
