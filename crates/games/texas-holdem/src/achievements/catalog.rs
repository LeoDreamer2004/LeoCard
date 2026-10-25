use super::{poker, squid};
use leocard_achievements::{AchievementDefinition, AchievementRegistry, combine};

const COUNT: usize = poker::DEFINITIONS.len() + squid::DEFINITIONS.len();

pub const DEFINITIONS: &[AchievementDefinition] =
    &combine::<COUNT>(&[poker::DEFINITIONS, squid::DEFINITIONS]);

pub fn register(registry: &mut AchievementRegistry) {
    registry.register(DEFINITIONS);
}
