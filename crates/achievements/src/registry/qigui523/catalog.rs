use super::{bronze, gold, silver};
use crate::registry::{AchievementDefinition, table::combine};

const COUNT: usize =
    bronze::DEFINITIONS.len() + silver::DEFINITIONS.len() + gold::DEFINITIONS.len();
pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] =
    &combine::<COUNT>(&[bronze::DEFINITIONS, silver::DEFINITIONS, gold::DEFINITIONS]);
