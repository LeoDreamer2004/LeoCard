use leocard_achievements::AchievementRegistry;
use std::sync::LazyLock;

/// Assemble the games supported by this application. Definitions and evaluation
/// stay in their respective crates; only the registration list lives here.
pub static ACHIEVEMENT_REGISTRY: LazyLock<AchievementRegistry> = LazyLock::new(|| {
    let mut registry = AchievementRegistry::default();
    leocard_mahjong::achievements::register(&mut registry);
    leocard_texas_holdem::achievements::register(&mut registry);
    leocard_shengji::achievements::register(&mut registry);
    leocard_uno::achievements::register(&mut registry);
    leocard_qigui523::achievements::register(&mut registry);
    leocard_mc_guess::achievements::register(&mut registry);
    leocard_doudizhu::achievements::register(&mut registry);
    registry
});
