use super::facts::action;
use leocard_achievements::{AchievementDefinition, award};

pub const DEFINITIONS: &[AchievementDefinition] = &[
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "combo_two",
        "连破",
        Bronze,
        "关系谜图中连击猜出一个条目",
        |e| action(e, |s| s.graph.is_some_and(|s| s.combo >= 2)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "combo_three",
        "连中三元",
        Bronze,
        "关系谜图中连击总共猜出三个条目",
        |e| action(e, |s| s.graph.is_some_and(|s| s.combo >= 3)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "combo_five",
        "劲爆五杀",
        Bronze,
        "关系谜图中连击总共猜出五个条目",
        |e| action(e, |s| s.graph.is_some_and(|s| s.combo >= 5)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "tag_neighbours",
        "探索新大陆",
        Bronze,
        "关系谜图中猜出一个周围只有标签节点的条目",
        |e| action(e, |s| s.graph.is_some_and(|s| s.only_tag_neighbours)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "graph_opening",
        "先发制人",
        Bronze,
        "关系谜图中作为首个猜词玩家上来猜出一个词条",
        |e| action(e, |s| s.graph.is_some_and(|s| s.opening_win)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "five_first_greens",
        "伟大的开源精神",
        Silver,
        "同一局关系谜图中，在五个不同节点首次给出全绿但未猜中的猜测",
        |e| action(e, |s| s.graph.is_some_and(|s| s.first_green_nodes >= 5)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "five_assists",
        "袁世凯在世",
        Silver,
        "同一局关系谜图中，在五个不同节点跟着他人全绿但未猜中的猜测后猜出答案",
        |e| action(e, |s| s.graph.is_some_and(|s| s.assisted_nodes >= 5)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "combo_sweep",
        "一键扫荡",
        Gold,
        "关系谜图中在没有任何词条被猜出的情况下，连续猜出所有词条",
        |e| action(e, |s| s.graph.is_some_and(|s| s.combo_sweep)),
        1,
        Lifetime
    ),
    award!(
        "leocard:minecraft_guess/",
        McGuess,
        "thousand_graph_wins",
        "小镇做题家",
        Gold,
        "累计在至少一千小局关系谜图中获得最高分（含并列）",
        |e| action(e, |s| s.graph.is_some_and(|s| s.round_win)),
        1000,
        Lifetime
    ),
];
