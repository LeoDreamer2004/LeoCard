use super::facts::{hand, whole_match};
use super::macros::doudizhu;
use leocard_achievements::{AchievementDefinition, award};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    doudizhu!(
        "five_thousand_farmer_wins",
        "人民当家作主",
        Gold,
        "作为农民累计赢下5000局",
        |event| hand(event, |s| !s.landlord && s.won),
        5000
    ),
    doudizhu!(
        "five_thousand_landlord_wins",
        "跨国资本财阀",
        Gold,
        "作为地主累计赢下5000局",
        |event| hand(event, |s| s.landlord && s.won),
        5000
    ),
    doudizhu!(
        "perfect_spring",
        "我xxx没有开挂",
        Gold,
        "整场每一小局均以春天或反春获胜",
        |event| whole_match(event, |s| s.perfect_spring)
    ),
    doudizhu!(
        "all_bomb_hand",
        "这里叫做xxx广场",
        Gold,
        "开打时全部手牌都能组成炸弹或王炸",
        |event| hand(event, |s| s.all_bomb_hand)
    ),
    doudizhu!(
        "five_hundred_points",
        "当年陈刀仔",
        Gold,
        "一场游戏累计净赢至少500分",
        |event| whole_match(event, |s| s.score >= 500)
    ),
    doudizhu!(
        "two_plays",
        "17张牌你能秒我？",
        Gold,
        "只用至多两手牌出完自己的全部手牌",
        |event| hand(event, |s| s.won
            && s.play_count > 0
            && s.play_count <= 2
            && s.finished_cards)
    ),
];
