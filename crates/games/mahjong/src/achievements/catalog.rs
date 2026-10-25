use super::{collection, fans, flowers, groups, milestones};
use leocard_achievements::{AchievementDefinition, AchievementRegistry, combine};

const COUNT: usize = fans::DEFINITIONS.len()
    + milestones::DEFINITIONS.len()
    + collection::DEFINITIONS.len()
    + groups::DEFINITIONS.len()
    + flowers::DEFINITIONS.len();
pub const DEFINITIONS: &[AchievementDefinition] = &combine::<COUNT>(&[
    fans::DEFINITIONS,
    milestones::DEFINITIONS,
    collection::DEFINITIONS,
    groups::DEFINITIONS,
    flowers::DEFINITIONS,
]);

pub fn register(registry: &mut AchievementRegistry) {
    registry.register(DEFINITIONS);
}
