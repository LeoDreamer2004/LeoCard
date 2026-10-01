macro_rules! award {
    ($namespace:literal, $category:ident, $id:literal, $title:literal, $tier:ident, $description:literal, $amount:expr, $target:expr, $scope:ident) => {
        AchievementDefinition {
            id: concat!($namespace, $id),
            category: AchievementCategory::$category,
            title: $title,
            tier: AchievementTier::$tier,
            description: $description,
            criteria: &[AchievementCriterion {
                id: "progress",
                amount: $amount,
                target: $target,
                scope: AchievementScope::$scope,
            }],
            requirements: &[&["progress"]],
        }
    };
}

pub(super) use award;
