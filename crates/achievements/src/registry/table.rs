use super::AchievementDefinition;

pub(super) const fn combine<const N: usize>(
    groups: &[&[AchievementDefinition]],
) -> [AchievementDefinition; N] {
    let mut definitions = [groups[0][0]; N];
    let mut index = 0;
    let mut group = 0;
    while group < groups.len() {
        let mut item = 0;
        while item < groups[group].len() {
            definitions[index] = groups[group][item];
            index += 1;
            item += 1;
        }
        group += 1;
    }
    assert!(index == N);
    definitions
}
