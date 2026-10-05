use leocard_texas_holdem::{
    TexasHoldemCard, TexasHoldemHandCategory, TexasHoldemRank, TexasHoldemRuleSet, TexasHoldemSuit,
};
use std::cmp::Reverse;

pub(super) struct HandGuideEntry {
    pub category: TexasHoldemHandCategory,
    pub description: &'static str,
    pub example: [TexasHoldemCard; 5],
}

pub(super) fn ordered_entries(rules: &TexasHoldemRuleSet) -> Vec<&'static HandGuideEntry> {
    let mut entries = ENTRIES.iter().collect::<Vec<_>>();
    entries.sort_by_key(|entry| Reverse(entry.category.strength(rules)));
    entries
}

const ENTRIES: [HandGuideEntry; 10] = [
    HandGuideEntry {
        category: TexasHoldemHandCategory::RoyalFlush,
        description: "同一花色的 A、K、Q、J、10，是最大的同花顺。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::King),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Jack),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ten),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::StraightFlush,
        description: "同一花色的五张连续点数牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Jack),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Ten),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Nine),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Eight),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::FourOfAKind,
        description: "四张相同点数的牌，另加一张其他牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::King),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::FullHouse,
        description: "三张相同点数的牌，加上另一组对子。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::King),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::King),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::King),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Queen),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::Flush,
        description: "五张同一花色、点数不连续的牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Jack),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Nine),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Eight),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Six),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::Straight,
        description: "五张连续点数、并非全部同花色的牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ten),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Nine),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Eight),
            TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Seven),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Six),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::ThreeOfAKind,
        description: "三张相同点数的牌，另加两张不同点数的牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Nine),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::TwoPair,
        description: "两组不同点数的对子，另加一张其他牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Jack),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Jack),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Eight),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Eight),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Ace),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::OnePair,
        description: "一组对子，另加三张不同点数的牌。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ten),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::Ten),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Eight),
        ],
    },
    HandGuideEntry {
        category: TexasHoldemHandCategory::HighCard,
        description: "没有构成以上任何牌型，比较单张点数。",
        example: [
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Ace),
            TexasHoldemCard::new(TexasHoldemSuit::Heart, TexasHoldemRank::King),
            TexasHoldemCard::new(TexasHoldemSuit::Club, TexasHoldemRank::Queen),
            TexasHoldemCard::new(TexasHoldemSuit::Diamond, TexasHoldemRank::Nine),
            TexasHoldemCard::new(TexasHoldemSuit::Spade, TexasHoldemRank::Seven),
        ],
    },
];
