use super::{facts::matches, macros::qigui523};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::qigui523) const DEFINITIONS: &[AchievementDefinition] = &[
    qigui523!(
        "first_blood",
        "第一滴血",
        Bronze,
        "在一局中赢下第一手并获得分数",
        |event| matches(event, |s| s.progress.first_trick_with_points),
        1
    ),
    qigui523!(
        "first_win",
        "旗开得胜",
        Bronze,
        "获得一场游戏胜利",
        |event| matches(event, |s| s.won),
        1
    ),
    qigui523!(
        "diamond_four",
        "大发慈悲",
        Bronze,
        "单打出一张方块4",
        |event| matches(event, |s| s.diamond_four),
        1
    ),
    qigui523!(
        "spade_seven_follow",
        "七进七出",
        Bronze,
        "在允许同类型贴牌时用黑桃7压下另一张黑桃7",
        |event| matches(event, |s| s.spade_seven_follow),
        1
    ),
    qigui523!(
        "consecutive_pairs",
        "这不叫拖拉机！",
        Bronze,
        "打出连对",
        |event| matches(event, |s| s.progress.consecutive_pair_plays > 0),
        1
    ),
    qigui523!(
        "bomb",
        "请温柔一点",
        Bronze,
        "打出炸弹",
        |event| matches(event, |s| s.progress.bomb_plays
            + s.progress.heaven_bomb_plays
            > 0),
        1
    ),
    qigui523!(
        "heaven_bomb",
        "天王降临",
        Bronze,
        "打出天王炸",
        |event| matches(event, |s| s.progress.heaven_bomb_plays > 0),
        1
    ),
    qigui523!(
        "airplane",
        "华丽起飞",
        Bronze,
        "打出飞机",
        |event| matches(event, |s| s.progress.airplane_plays > 0),
        1
    ),
    qigui523!(
        "five_silent_tricks",
        "乌龟不出头",
        Bronze,
        "连续5手不打出任何牌",
        |event| matches(event, |s| s.progress.max_silent_tricks >= 5),
        1
    ),
    qigui523!(
        "late_bomb",
        "图穷匕见",
        Bronze,
        "在游戏最后没有剩余摸牌时打出炸弹或以上的牌型",
        |event| matches(event, |s| s.late_bomb),
        1
    ),
    qigui523!(
        "scoring_bomb_win",
        "你这太赖了",
        Bronze,
        "打出一个用5/10/K组成的炸弹并赢下此手",
        |event| matches(event, |s| s.progress.won_scoring_bomb),
        1
    ),
    qigui523!(
        "straight_from_four",
        "没人喜欢小牌",
        Bronze,
        "打出一个至少包含5张且从4开始的顺子",
        |event| matches(event, |s| s.straight_from_four),
        1
    ),
    qigui523!(
        "five_unanswered_tricks",
        "你们休息一会",
        Bronze,
        "连续打出5轮牌无任何玩家响应",
        |event| matches(event, |s| s.progress.max_unanswered_tricks >= 5),
        1
    ),
    qigui523!(
        "full_high_hand",
        "已经不知道该打什么了",
        Bronze,
        "在手牌数满的情况下所有的牌都不小于3",
        |event| matches(event, |s| s.progress.full_high_hand),
        1
    ),
    qigui523!(
        "comeback",
        "笑到最后才是赢",
        Bronze,
        "通过最后一手牌先出完逆转拿下第一名",
        |event| matches(event, |s| s.comeback),
        1
    ),
];
