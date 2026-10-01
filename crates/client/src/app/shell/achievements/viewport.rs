//! 图鉴页面重建时读取实际分类滚动位置。
use super::AchievementCategoryScroll;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
#[derive(SystemParam)]
pub(crate) struct AchievementPageViewport<'w, 's> {
    scroll: Query<'w, 's, &'static ScrollPosition, With<AchievementCategoryScroll>>,
}
impl AchievementPageViewport<'_, '_> {
    pub(crate) fn offset(&self) -> f32 {
        self.scroll.single().map_or(0.0, |position| position.y)
    }
}
