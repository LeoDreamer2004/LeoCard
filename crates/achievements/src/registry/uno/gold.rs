use super::{facts::matches, macros::uno};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::uno) const DEFINITIONS: &[AchievementDefinition] = &[
    uno!(
        "one_color_win",
        "这就是清一色吧？",
        Gold,
        "普通模式中只出同一种颜色并获胜，允许黑色万能牌",
        |event| matches(event, |s| s.rules.is_classic()
            && s.won
            && s.progress.colors.count_ones() == 1),
        1
    ),
    uno!(
        "ten_thousand",
        "万局不倒",
        Gold,
        "累计进行 10000 局",
        |event| matches(event, |s| s.completed_game),
        10000
    ),
    uno!(
        "full_draw_chain",
        "罚牌大满贯",
        Gold,
        "普通模式中一次受到全部 8 张 +2 和 4 张 +4 的叠加罚牌",
        |event| matches(event, |s| s.rules.is_classic()
            && s.progress.full_draw_chain),
        1
    ),
    uno!(
        "ten_uno_turns",
        "真的好吵啊",
        Gold,
        "一局中连续 10 个自己的正常回合都喊出 UNO",
        |event| matches(event, |s| s.progress.max_uno_call_run >= 10),
        1
    ),
    uno!(
        "merciful_win",
        "慈悲为怀",
        Gold,
        "No Mercy 中不使用任何罚牌牌获胜",
        |event| matches(event, |s| s.rules.is_no_mercy()
            && s.won
            && !s.progress.played_penalty),
        1
    ),
];
