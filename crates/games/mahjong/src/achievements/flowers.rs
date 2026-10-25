use super::events::MahjongAchievementEvent;
use super::facts::MahjongFacts;
use leocard_achievements::{AchievementDefinition, AchievementTrigger, award};

pub(super) const DEFINITIONS: &[AchievementDefinition] = &[
    award!(
        "leocard:mahjong/",
        Mahjong,
        "three_flowers_win",
        "三花聚鼎",
        Bronze,
        "和牌时至少含有三张花牌",
        |event| u64::from(winning_flowers(event) >= 3),
        1,
        Lifetime
    ),
    award!(
        "leocard:mahjong/",
        Mahjong,
        "five_hundred_winning_flowers",
        "花开富贵",
        Silver,
        "总计和出500张花牌",
        winning_flowers,
        500,
        Lifetime
    ),
    award!(
        "leocard:mahjong/",
        Mahjong,
        "two_thousand_drawn_flowers",
        "花前月下",
        Silver,
        "总计摸到2000张花牌（包含起手补花与流局中收下的花牌）",
        |event| u64::from(matches!(
            event.game::<MahjongAchievementEvent>(),
            Some((player, MahjongAchievementEvent::FlowerReplaced { player: actor }))
                if player == *actor
        )),
        2000,
        Lifetime
    ),
    award!(
        "leocard:mahjong/",
        Mahjong,
        "eight_flowers_win",
        "四季如春",
        Gold,
        "和牌时拥有全部八张花牌",
        |event| u64::from(winning_flowers(event) == 8),
        1,
        Lifetime
    ),
];

fn winning_flowers(trigger: &AchievementTrigger) -> u64 {
    MahjongFacts::from_trigger(trigger).map_or(0, |facts| {
        facts
            .win()
            .map_or(0, |win| u64::from(win.score.flower_points))
    })
}
