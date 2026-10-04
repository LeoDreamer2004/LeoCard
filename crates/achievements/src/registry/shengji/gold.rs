use super::{facts::*, macros::shengji};
use crate::registry::{macros::award, types::*};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    shengji!(
        "eight_jokers",
        "八王之乱",
        Gold,
        "4副牌中拿到全部8个王",
        |event| hand(event, |facts| facts.deck_count == 4
            && facts.opening_hand.joker_count == 8),
        1
    ),
    shengji!(
        "all_trump",
        "我的牌闪闪发光",
        Gold,
        "开打前完整手牌中所有牌都是主牌",
        |event| hand(event, |facts| facts.opening_hand.card_count > 0
            && facts.opening_hand.card_count == facts.opening_hand.trump_count),
        1
    ),
    shengji!(
        "throw_twenty",
        "屏幕要装不下了",
        Gold,
        "甩牌超过20张",
        |event| thrown_over(event, 20),
        1
    ),
    shengji!(
        "kitty_takeover",
        "谁笑到最后",
        Gold,
        "坐闲在一整局中未获一分，但是成功抠底直接上台",
        |event| hand(event, |facts| attack_promotes(facts, 0)
            && facts.result.trick_points == 0
            && facts.result.penalty_adjustment == 0
            && facts.result.kitty_points > 0
            && facts.result.kitty_multiplier > 0),
        1
    ),
    shengji!(
        "three_shutouts",
        "这样欺负闲家",
        Gold,
        "连续三盘让闲家大光",
        |event| analyzed(event, |facts, game| shutout(facts)
            && game.consecutive_dealer_shutouts[usize::from(facts.player.team().0)]
                >= 3),
        1
    ),
    shengji!(
        "ten_thousand",
        "万匹马达",
        Gold,
        "累计完成10000局",
        |event| hand(event, |_| true),
        10_000
    ),
    shengji!(
        "exhausted_redeal",
        "想找个庄家怎么这么难",
        Gold,
        "双方断电扳底后，底牌全翻开也无法确定庄家导致重新游戏",
        exhausted_redeal,
        1
    ),
];
