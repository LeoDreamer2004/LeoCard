use super::{
    facts::{amount, matches},
    macros::qigui523,
};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::qigui523) const DEFINITIONS: &[AchievementDefinition] = &[
    qigui523!(
        "six_straights",
        "六六大顺",
        Silver,
        "连续在自己回合出六次顺子",
        |event| matches(event, |s| s.progress.max_straight_run >= 6),
        1
    ),
    qigui523!(
        "all_points",
        "尽饱私囊",
        Silver,
        "一局中拿下所有分牌",
        |event| matches(event, |s| s.all_points),
        1
    ),
    qigui523!(
        "five_bombs",
        "狂轰滥炸",
        Silver,
        "一局中打出不少于5个炸弹",
        |event| matches(event, |s| s.progress.bomb_plays
            + s.progress.heaven_bomb_plays
            >= 5),
        1
    ),
    qigui523!(
        "thousand_fives",
        "五福临门",
        Silver,
        "累计抓到5分1000次",
        |event| amount(event, |s| s.captured_fives),
        1000
    ),
    qigui523!(
        "thousand_tens",
        "十全十美",
        Silver,
        "累计抓到10/K共1000次",
        |event| amount(event, |s| s.captured_tens_and_kings),
        1000
    ),
    qigui523!(
        "heaven_over_heaven",
        "你是银角大王，但我是金角大王",
        Silver,
        "用天王炸大过别人的天王炸",
        |event| matches(event, |s| s.heaven_over_heaven),
        1
    ),
    qigui523!(
        "twelve_cards",
        "请勿在牌桌上扔牌",
        Silver,
        "一次性打出不少于12张牌",
        |event| matches(event, |s| s.played_count >= 12),
        1
    ),
    qigui523!(
        "hundred_point_trick",
        "我只好笑纳了",
        Silver,
        "赢下不少于100分的一手牌",
        |event| matches(event, |s| s.progress.max_trick_points >= 100),
        1
    ),
];
