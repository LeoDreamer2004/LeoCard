use super::{economy, general};
use crate::{AchievementDefinition, combine};

const COUNT: usize = general::DEFINITIONS.len() + economy::DEFINITIONS.len();

pub(in crate::registry) const DEFINITIONS: &[AchievementDefinition] =
    &combine::<COUNT>(&[general::DEFINITIONS, economy::DEFINITIONS]);
