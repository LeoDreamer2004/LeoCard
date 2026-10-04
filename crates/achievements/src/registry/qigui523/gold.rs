use super::{
    facts::{amount, matches},
    macros::qigui523,
};
use crate::registry::{macros::award, types::*};

pub(in crate::registry::qigui523) const DEFINITIONS: &[AchievementDefinition] = &[
    qigui523!(
        "ten_thousand_points",
        "万分归宗",
        Gold,
        "累计抓分10000",
        |event| amount(event, |s| s.captured_points),
        10000
    ),
    qigui523!(
        "full_hand_play",
        "制衡联合，稳而不乱",
        Gold,
        "在游戏规则手牌数不少于8的情况下满手牌一次性全部打出",
        |event| matches(event, |s| s.full_hand_play),
        1
    ),
    qigui523!(
        "bomb_revenge",
        "冤家路窄",
        Gold,
        "一手中，别家用K炸大过10炸，自己再用5炸大过K炸",
        |event| matches(event, |s| s.bomb_revenge),
        1
    ),
    qigui523!(
        "uncontested_win",
        "这是单人游戏吗？",
        Gold,
        "在其他玩家全部都不出牌的情况下赢下一整局",
        |event| matches(event, |s| s.uncontested_win),
        1
    ),
    qigui523!(
        "ten_bombs",
        "超级炸弹兵",
        Gold,
        "一局中打出不少于10个炸弹",
        |event| matches(event, |s| s.progress.bomb_plays
            + s.progress.heaven_bomb_plays
            >= 10),
        1
    ),
];
