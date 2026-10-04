use super::{
    facts::{amount, matches},
    macros::uno,
};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::uno) const DEFINITIONS: &[AchievementDefinition] = &[
    uno!(
        "call_uno",
        "胜利宣言",
        Bronze,
        "喊出一次 UNO",
        |event| matches(event, |s| s.progress.uno_calls > 0),
        1
    ),
    uno!(
        "report_uno",
        "下次注意点",
        Bronze,
        "检举一次他人未喊 UNO",
        |event| matches(event, |s| s.progress.uno_reports > 0),
        1
    ),
    uno!(
        "pair_win",
        "不喊 UNO 也能赢",
        Bronze,
        "在允许抢出时最后一手打出一对赢得游戏",
        |event| matches(event, |s| s.rules.jump_in_enabled()
            && s.won
            && s.progress.finished_with_pair),
        1
    ),
    uno!(
        "all_numbers",
        "数字齐步",
        Bronze,
        "一局中出齐 0-9",
        |event| matches(event, |s| s.progress.numbers & 0x03ff == 0x03ff),
        1
    ),
    uno!(
        "dark_win",
        "月之暗面",
        Bronze,
        "Flip 中在暗面获胜",
        |event| matches(event, |s| s.rules.is_flip() && s.won && s.dark_side),
        1
    ),
    uno!(
        "eliminated",
        "好走不送",
        Bronze,
        "No Mercy 中被淘汰",
        |event| matches(event, |s| s.rules.is_no_mercy() && s.eliminated),
        1
    ),
    uno!(
        "sole_survivor",
        "唯我独尊",
        Bronze,
        "No Mercy 中其他所有玩家被淘汰，自己成为唯一幸存者",
        |event| matches(event, |s| s.rules.is_no_mercy()
            && s.won
            && s.all_opponents_eliminated),
        1
    ),
    uno!(
        "draw_backfire",
        "搬石砸脚",
        Bronze,
        "自己先打出罚牌，叠加后最终传回自己",
        |event| matches(event, |s| s.progress.draw_chain_returned),
        1
    ),
    uno!(
        "three_skips",
        "小黑屋",
        Bronze,
        "在允许禁手堆叠时被一次性禁手三轮及以上",
        |event| matches(event, |s| s.rules.action_stacking_enabled()
            && s.progress.max_skip_batch >= 3),
        1
    ),
    uno!(
        "jump_in",
        "叨扰一下",
        Bronze,
        "抢出成功一次",
        |event| matches(event, |s| s.progress.jump_ins > 0),
        1
    ),
    uno!(
        "twenty_four",
        "寸止的艺术",
        Bronze,
        "No Mercy 中手牌达到 24 张",
        |event| matches(event, |s| s.rules.is_no_mercy()
            && s.progress.reached_twenty_four),
        1
    ),
    uno!(
        "challenge",
        "还想骗过我",
        Bronze,
        "质疑万能牌成功",
        |event| matches(event, |s| s.progress.successful_challenges > 0),
        1
    ),
    uno!(
        "no_penalty_win",
        "一路顺风",
        Bronze,
        "一局中未被罚牌并获胜",
        |event| matches(event, |s| s.won && !s.progress.has_penalty),
        1
    ),
    uno!(
        "jump_reverse",
        "搞不清该往哪里走",
        Bronze,
        "抢出一张反转牌",
        |event| matches(event, |s| s.progress.jumped_reverse),
        1
    ),
    uno!(
        "color_gift",
        "感谢馈赠",
        Bronze,
        "仅剩一张牌时，接上上家修改的颜色并获胜",
        |event| matches(event, |s| s.won && s.progress.finished_with_color_gift),
        1
    ),
    uno!(
        "hand_swaps",
        "你的是我的，我的是你的",
        Bronze,
        "No Mercy 中自己通过 0 或 7 完成换牌不少于 10 次",
        |event| amount(event, |s| if s.rules.is_no_mercy() {
            s.hand_swaps
        } else {
            0
        }),
        10
    ),
];
