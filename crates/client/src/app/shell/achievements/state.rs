use bevy::prelude::*;
pub(crate) use leocard_client::AchievementCategory;

pub(super) const ACCENT: Color = Color::srgb(0.77, 0.72, 1.0);
pub(super) const GOLD: Color = Color::srgb(0.91, 0.78, 0.55);

pub(super) const CATEGORIES: [(AchievementCategory, &str); 6] = [
    (AchievementCategory::Mahjong, "四方争雄"),
    (AchievementCategory::QiGui523, "七擒七纵"),
    (AchievementCategory::TexasHoldem, "运筹帷幄"),
    (AchievementCategory::Shengji, "步步高升"),
    (AchievementCategory::Uno, "千变万化"),
    (AchievementCategory::Personal, "独善其身"),
];

#[derive(Component)]
pub(crate) struct AchievementsScroll;

#[derive(Component)]
pub(crate) struct AchievementCategoryScroll;

#[derive(Component)]
pub(crate) struct AchievementCategoryButton {
    pub(super) selected: bool,
    pub(super) emblem: Entity,
}

#[derive(Default)]
pub(crate) struct AchievementsUiState {
    pub open: bool,
    pub category: AchievementCategory,
}
