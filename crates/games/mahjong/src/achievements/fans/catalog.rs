use super::{high, repeated, standard};
use leocard_achievements::{AchievementDefinition, combine};

const COUNT: usize =
    high::DEFINITIONS.len() + standard::DEFINITIONS.len() + repeated::DEFINITIONS.len();

pub(in crate::achievements) const DEFINITIONS: &[AchievementDefinition] = &combine::<COUNT>(&[
    high::DEFINITIONS,
    standard::DEFINITIONS,
    repeated::DEFINITIONS,
]);
