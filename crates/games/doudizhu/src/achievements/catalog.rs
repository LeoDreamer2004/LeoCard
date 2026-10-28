use super::{bronze, gold, silver};
use leocard_achievements::{AchievementDefinition, AchievementRegistry, combine};

const COUNT: usize =
    bronze::DEFINITIONS.len() + silver::DEFINITIONS.len() + gold::DEFINITIONS.len();
pub const DEFINITIONS: &[AchievementDefinition] =
    &combine::<COUNT>(&[bronze::DEFINITIONS, silver::DEFINITIONS, gold::DEFINITIONS]);

pub fn register(registry: &mut AchievementRegistry) {
    registry.register(DEFINITIONS);
}
