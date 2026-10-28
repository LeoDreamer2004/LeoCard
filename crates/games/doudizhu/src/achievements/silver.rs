use super::facts::{airplane_groups, hand, play};
use super::macros::doudizhu;
use crate::PlayKind;
use leocard_achievements::{AchievementDefinition, award};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    doudizhu!(
        "three_bombs_in_trick",
        "不好意思有点疼",
        Silver,
        "在同一轮争夺中自己打出至少三个炸弹或王炸",
        |event| play(event, |s| s.trick_bombs >= 3)
    ),
    doudizhu!(
        "full_straight",
        "通天顺",
        Silver,
        "打出从3到A的十二张顺子",
        |event| play(event, |s| s.kind == PlayKind::Straight
            && s.card_count == 12
            && s.rank == 11)
    ),
    doudizhu!(
        "seven_pairs",
        "姊妹对",
        Silver,
        "打出至少七组的连对",
        |event| play(event, |s| s.kind == PlayKind::Pairs && s.card_count >= 14)
    ),
    doudizhu!(
        "four_airplane",
        "神鹰哥",
        Silver,
        "打出至少四组的飞机（可带单张或对子）",
        |event| play(event, |s| airplane_groups(s) >= 4)
    ),
    doudizhu!(
        "fifty_points",
        "土块",
        Silver,
        "在一局中赢下至少50分",
        |event| hand(event, |s| s.score >= 50)
    ),
    doudizhu!(
        "three_consecutive_bombs",
        "开始你的炸弹秀",
        Silver,
        "连续三个自己的回合均打出炸弹或王炸，中途不能不出",
        |event| play(event, |s| s.consecutive_bombs >= 3)
    ),
];
