use super::{classic, discovery, graph};
use leocard_achievements::{AchievementDefinition, AchievementRegistry, combine};

const COUNT: usize =
    classic::DEFINITIONS.len() + discovery::DEFINITIONS.len() + graph::DEFINITIONS.len();
pub const DEFINITIONS: &[AchievementDefinition] = &combine::<COUNT>(&[
    classic::DEFINITIONS,
    discovery::DEFINITIONS,
    graph::DEFINITIONS,
]);

pub fn register(registry: &mut AchievementRegistry) {
    registry.register(DEFINITIONS);
    registry.register_derivation(super::facts::derived_triggers);
}
