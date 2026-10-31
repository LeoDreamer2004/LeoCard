use super::DoudizhuAchievementFacts;
use super::facts::{airplane_groups, facts, hand, play, whole_match};
use super::macros::doudizhu;
use crate::{Mode, PlayKind};
use leocard_achievements::{AchievementDefinition, award};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    doudizhu!(
        "landlord_win",
        "飞扬跋扈",
        Bronze,
        "作为地主赢下一局",
        |event| hand(event, |s| s.landlord && s.won)
    ),
    doudizhu!(
        "farmer_win",
        "土地革命",
        Bronze,
        "作为农民赢下一局",
        |event| hand(event, |s| !s.landlord && s.won)
    ),
    doudizhu!(
        "three_player_bomb",
        "请温柔一点",
        Bronze,
        "在三人斗地主中打出炸弹或王炸",
        |event| play(event, |s| s.mode == Mode::Three
            && matches!(s.kind, PlayKind::Bomb | PlayKind::Rocket))
    ),
    doudizhu!("airplane", "腾空万里", Bronze, "打出飞机", |event| {
        play(event, |s| airplane_groups(s) >= 2)
    }),
    doudizhu!(
        "higher_airplane",
        "飞的更高",
        Bronze,
        "用飞机压过另一名玩家的飞机",
        |event| play(event, |s| s.beats_airplane)
    ),
    doudizhu!(
        "landlord_spring",
        "你们怎么都不出牌",
        Bronze,
        "作为地主以春天赢下一局",
        |event| hand(event, |s| s.landlord && s.won && s.spring)
    ),
    doudizhu!(
        "farmer_spring",
        "谁才是真的地主",
        Bronze,
        "作为农民以反春赢下一局",
        |event| hand(event, |s| !s.landlord && s.won && s.spring)
    ),
    doudizhu!(
        "lead_six",
        "单走一个六",
        Bronze,
        "领牌时打出单张6",
        |event| play(event, |s| s.is_lead
            && s.kind == PlayKind::Single
            && s.rank == 3)
    ),
    doudizhu!(
        "bomb_finish",
        "得得得得得得得得得",
        Bronze,
        "用炸弹或王炸出完最后一手牌获胜",
        |event| hand(event, |s| s.won && s.finishing_bomb)
    ),
    doudizhu!(
        "unavailable_exchange",
        "可惜换不得",
        Bronze,
        "开打时手持3、4、5、6各一张且没有7",
        |event| hand(event, |s| s.unavailable_exchange)
    ),
    doudizhu!(
        "flower_bomb",
        "给阿姨倒一杯卡布奇诺",
        Bronze,
        "在农民队友的回合向其送花，随后队友在该回合打出炸弹或王炸",
        |event| u64::from(matches!(
            facts(event),
            Some(DoudizhuAchievementFacts::FlowerBomb)
        ))
    ),
    doudizhu!(
        "first_to_last",
        "玩游戏要啸着玩...",
        Bronze,
        "最后一局前排名第一，整场结束时跌至最后一名",
        |event| whole_match(event, |s| s.first_to_last)
    ),
];
