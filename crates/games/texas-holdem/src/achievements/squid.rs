use super::facts::squid;
use leocard_achievements::{AchievementDefinition, award};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    award!(
        "leocard:texas/",
        TexasHoldem,
        "first_deepseek_squid",
        "吃白饭的肥鱼",
        Bronze,
        "获得一次DeepSeek牌鱿鱼",
        |event| squid(event, |statistics| statistics.deepseek),
        1,
        Lifetime
    ),
    award!(
        "leocard:texas/",
        TexasHoldem,
        "three_squid_in_round",
        "鱿鱼串串香",
        Bronze,
        "在一轮重置内获得至少三个鱿鱼",
        |event| squid(event, |statistics| statistics.owned >= 3),
        1,
        Lifetime
    ),
    award!(
        "leocard:texas/",
        TexasHoldem,
        "fifty_deepseek_squid",
        "肥鱼的恩惠",
        Silver,
        "累计获得50次DeepSeek牌鱿鱼",
        |event| squid(event, |statistics| statistics.deepseek),
        50,
        Lifetime
    ),
    award!(
        "leocard:texas/",
        TexasHoldem,
        "five_squid_in_round",
        "水族馆管理员",
        Silver,
        "在一轮重置内获得至少五个鱿鱼",
        |event| squid(event, |statistics| statistics.owned >= 5),
        1,
        Lifetime
    ),
    award!(
        "leocard:texas/",
        TexasHoldem,
        "five_deepseek_squid_in_round",
        "超级哦鲸鲸",
        Gold,
        "在一轮重置内获得五个DeepSeek牌鱿鱼",
        |event| squid(event, |statistics| statistics.deepseek_owned >= 5),
        1,
        Lifetime
    ),
];
