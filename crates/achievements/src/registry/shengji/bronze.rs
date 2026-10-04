use super::{facts::*, macros::shengji};
use crate::registry::{macros::award, types::*};
use leocard_shengji::Component;

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    shengji!("counter", "官大一级", Bronze, "进行一次反主", counter, 1),
    shengji!(
        "no_trump",
        "我并不喜欢那么多花色",
        Bronze,
        "亮或者反一次无主",
        |event| declaration(event, |facts| facts.trump.trump_suit().is_none()),
        1
    ),
    shengji!(
        "attack_win",
        "反客为主",
        Bronze,
        "坐闲赢下一盘",
        |event| hand(event, |facts| attack_promotes(facts, 0)),
        1
    ),
    shengji!(
        "throw_eight",
        "出口成章",
        Bronze,
        "甩牌超过8张",
        |event| thrown_over(event, 8),
        1
    ),
    shengji!(
        "two_deck_ten",
        "二副小胜",
        Bronze,
        "完成10盘2副牌",
        |event| completed(event, 2),
        10
    ),
    shengji!(
        "three_deck_ten",
        "三副小胜",
        Bronze,
        "完成10盘3副牌",
        |event| completed(event, 3),
        10
    ),
    shengji!(
        "four_deck_ten",
        "四副小胜",
        Bronze,
        "完成10盘4副牌",
        |event| completed(event, 4),
        10
    ),
    shengji!(
        "clean_bottom",
        "干净又卫生",
        Bronze,
        "坐庄底牌没有埋分",
        |event| hand(event, |facts| {
            facts.player == facts.result.dealer
                && facts
                    .burial
                    .is_some_and(|burial| burial.card_count > 0 && burial.points == 0)
        }),
        1
    ),
    shengji!(
        "shutout",
        "剃光头",
        Bronze,
        "坐庄让闲家大光",
        |event| hand(event, shutout),
        1
    ),
    shengji!(
        "copy_bottom",
        "让我看看你藏的怎么样",
        Bronze,
        "进行一次抄底",
        copy_bottom,
        1
    ),
    shengji!(
        "attack_three",
        "我不客气了",
        Bronze,
        "坐闲一局得分至少升三级",
        |event| hand(event, |facts| attack_promotes(facts, 3)),
        1
    ),
    shengji!(
        "long_tractor",
        "大户人家",
        Bronze,
        "打出至少4连的拖拉机",
        |event| play(event, |facts, _| {
            facts.play.components.iter().any(|component| matches!(component, Component::Tractor { pair_count, .. } if *pair_count >= 4))
        }),
        1
    ),
    shengji!(
        "structured_kitty",
        "你一定很紧张吧",
        Bronze,
        "坐闲用至少拖拉机的牌型抠底，且底上有分",
        |event| hand(event, captured_with_structure),
        1
    ),
    shengji!(
        "first_cut",
        "没想到吧",
        Bronze,
        "在游戏第一手就杀掉庄家的牌",
        |event| hand(event, |facts| facts.first_trick_cut_dealer),
        1
    ),
];
