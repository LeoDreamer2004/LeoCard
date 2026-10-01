//! 成就图鉴与分类浏览。

mod actions;
mod categories;
mod entries;
mod input;
mod page;
mod plugin;
mod state;

pub(crate) use input::update_achievement_category_hover;
pub(crate) use page::AchievementsPage;
pub(crate) use plugin::AchievementsPagePlugin;
pub(crate) use state::{AchievementCategory, AchievementCategoryScroll, AchievementsUiState};

pub(crate) use actions::AchievementUiAction;

mod entry;
pub(crate) use entry::AchievementEntry;

mod viewport;
pub(crate) use viewport::AchievementPageViewport;
