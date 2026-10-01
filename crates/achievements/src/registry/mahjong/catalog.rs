use super::super::{AchievementDefinition, table::combine};
use super::{collection, fans, milestones};

const COUNT: usize =
    fans::DEFINITIONS.len() + milestones::DEFINITIONS.len() + collection::DEFINITIONS.len();
pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] = &combine::<COUNT>(&[
    fans::DEFINITIONS,
    milestones::DEFINITIONS,
    collection::DEFINITIONS,
]);
