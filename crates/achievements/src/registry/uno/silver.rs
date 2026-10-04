use super::{
    facts::{amount, matches},
    macros::uno,
};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::uno) const DEFINITIONS: &[AchievementDefinition] = &[
    uno!(
        "wild_two_hundred",
        "万能变换",
        Silver,
        "累计打出万能牌 200 次",
        |event| amount(event, |s| s.wild_cards),
        200
    ),
    uno!(
        "hundred_jumps",
        "手速王",
        Silver,
        "抢出成功 100 次",
        |event| amount(event, |s| s.jump_ins),
        100
    ),
    uno!(
        "no_draw_win",
        "无懈可击",
        Silver,
        "一局中从未抓牌并获胜",
        |event| matches(event, |s| s.won && !s.progress.has_drawn),
        1
    ),
    uno!(
        "forgotten_uno_win",
        "暗度陈仓",
        Silver,
        "剩一张牌时忘记喊 UNO，未被检举并在下次出牌获胜",
        |event| matches(event, |s| s.won && s.progress.finished_without_uno),
        1
    ),
    uno!(
        "fifty_cards",
        "囤积居奇",
        Silver,
        "普通模式中手牌数达到 50 张",
        |event| matches(event, |s| s.rules.is_classic()
            && s.progress.peak_hand >= 50),
        1
    ),
    uno!(
        "lucky_elimination",
        "某种程度上很幸运",
        Silver,
        "No Mercy 中手牌不多于 5 张时，非罚牌摸牌直接被淘汰",
        |event| matches(event, |s| s.rules.is_no_mercy()
            && s.progress.lucky_elimination),
        1
    ),
];
