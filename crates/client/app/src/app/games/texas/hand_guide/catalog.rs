use leocard_texas_holdem::{TexasCard, TexasHandCategory, TexasRank, TexasRuleSet, TexasSuit};
use std::cmp::Reverse;

pub(super) struct HandGuideEntry {
    pub category: TexasHandCategory,
    pub description: &'static str,
    pub example: [TexasCard; 5],
}

pub(super) fn ordered_entries(rules: &TexasRuleSet) -> Vec<&'static HandGuideEntry> {
    let mut entries = ENTRIES.iter().collect::<Vec<_>>();
    entries.sort_by_key(|entry| Reverse(entry.category.strength(rules)));
    entries
}

const ENTRIES: [HandGuideEntry; 10] = [
    HandGuideEntry {
        category: TexasHandCategory::RoyalFlush,
        description: "同一花色的 A、K、Q、J、10，是最大的同花顺。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
            TexasCard::new(TexasSuit::Spade, TexasRank::King),
            TexasCard::new(TexasSuit::Spade, TexasRank::Queen),
            TexasCard::new(TexasSuit::Spade, TexasRank::Jack),
            TexasCard::new(TexasSuit::Spade, TexasRank::Ten),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::StraightFlush,
        description: "同一花色的五张连续点数牌。",
        example: [
            TexasCard::new(TexasSuit::Heart, TexasRank::Queen),
            TexasCard::new(TexasSuit::Heart, TexasRank::Jack),
            TexasCard::new(TexasSuit::Heart, TexasRank::Ten),
            TexasCard::new(TexasSuit::Heart, TexasRank::Nine),
            TexasCard::new(TexasSuit::Heart, TexasRank::Eight),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::FourOfAKind,
        description: "四张相同点数的牌，另加一张其他牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
            TexasCard::new(TexasSuit::Heart, TexasRank::Ace),
            TexasCard::new(TexasSuit::Club, TexasRank::Ace),
            TexasCard::new(TexasSuit::Diamond, TexasRank::Ace),
            TexasCard::new(TexasSuit::Spade, TexasRank::King),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::FullHouse,
        description: "三张相同点数的牌，加上另一组对子。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::King),
            TexasCard::new(TexasSuit::Heart, TexasRank::King),
            TexasCard::new(TexasSuit::Club, TexasRank::King),
            TexasCard::new(TexasSuit::Spade, TexasRank::Queen),
            TexasCard::new(TexasSuit::Heart, TexasRank::Queen),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::Flush,
        description: "五张同一花色、点数不连续的牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
            TexasCard::new(TexasSuit::Spade, TexasRank::Jack),
            TexasCard::new(TexasSuit::Spade, TexasRank::Nine),
            TexasCard::new(TexasSuit::Spade, TexasRank::Eight),
            TexasCard::new(TexasSuit::Spade, TexasRank::Six),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::Straight,
        description: "五张连续点数、并非全部同花色的牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ten),
            TexasCard::new(TexasSuit::Heart, TexasRank::Nine),
            TexasCard::new(TexasSuit::Club, TexasRank::Eight),
            TexasCard::new(TexasSuit::Diamond, TexasRank::Seven),
            TexasCard::new(TexasSuit::Spade, TexasRank::Six),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::ThreeOfAKind,
        description: "三张相同点数的牌，另加两张不同点数的牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Queen),
            TexasCard::new(TexasSuit::Heart, TexasRank::Queen),
            TexasCard::new(TexasSuit::Club, TexasRank::Queen),
            TexasCard::new(TexasSuit::Diamond, TexasRank::Ace),
            TexasCard::new(TexasSuit::Spade, TexasRank::Nine),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::TwoPair,
        description: "两组不同点数的对子，另加一张其他牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Jack),
            TexasCard::new(TexasSuit::Heart, TexasRank::Jack),
            TexasCard::new(TexasSuit::Spade, TexasRank::Eight),
            TexasCard::new(TexasSuit::Heart, TexasRank::Eight),
            TexasCard::new(TexasSuit::Club, TexasRank::Ace),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::OnePair,
        description: "一组对子，另加三张不同点数的牌。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ten),
            TexasCard::new(TexasSuit::Heart, TexasRank::Ten),
            TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
            TexasCard::new(TexasSuit::Club, TexasRank::Queen),
            TexasCard::new(TexasSuit::Diamond, TexasRank::Eight),
        ],
    },
    HandGuideEntry {
        category: TexasHandCategory::HighCard,
        description: "没有构成以上任何牌型，比较单张点数。",
        example: [
            TexasCard::new(TexasSuit::Spade, TexasRank::Ace),
            TexasCard::new(TexasSuit::Heart, TexasRank::King),
            TexasCard::new(TexasSuit::Club, TexasRank::Queen),
            TexasCard::new(TexasSuit::Diamond, TexasRank::Nine),
            TexasCard::new(TexasSuit::Spade, TexasRank::Seven),
        ],
    },
];
