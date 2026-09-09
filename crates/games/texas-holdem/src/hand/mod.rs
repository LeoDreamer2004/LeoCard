#[cfg(test)]
mod tests;

use crate::{TexasHoldemCard, TexasHoldemRank, TexasHoldemRuleSet};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasHoldemHandCategory {
    HighCard,
    OnePair,
    TwoPair,
    Straight,
    ThreeOfAKind,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvaluatedHand {
    category: TexasHoldemHandCategory,
    category_strength: u8,
    kickers: [u8; 5],
    cards: [TexasHoldemCard; 5],
}

impl EvaluatedHand {
    pub const fn category(self) -> TexasHoldemHandCategory {
        self.category
    }

    pub const fn kickers(self) -> [u8; 5] {
        self.kickers
    }

    pub const fn cards(self) -> [TexasHoldemCard; 5] {
        self.cards
    }

    /// 按房间规则比较两副已评估的牌。
    ///
    /// 标准规则使用完整的德州排序键；忽略踢脚牌时，只保留构成牌型的点数。
    pub fn cmp_with_rules(&self, other: &Self, rules: &TexasHoldemRuleSet) -> Ordering {
        if !rules.ignore_kickers {
            return self.cmp(other);
        }
        (
            category_strength(self.category, rules.short_deck),
            core_hand_ranks(self.category, self.kickers),
        )
            .cmp(&(
                category_strength(other.category, rules.short_deck),
                core_hand_ranks(other.category, other.kickers),
            ))
    }
}

const fn core_hand_ranks(category: TexasHoldemHandCategory, kickers: [u8; 5]) -> [u8; 5] {
    match category {
        TexasHoldemHandCategory::TwoPair | TexasHoldemHandCategory::FullHouse => {
            [kickers[0], kickers[1], 0, 0, 0]
        }
        TexasHoldemHandCategory::HighCard
        | TexasHoldemHandCategory::OnePair
        | TexasHoldemHandCategory::Straight
        | TexasHoldemHandCategory::ThreeOfAKind
        | TexasHoldemHandCategory::Flush
        | TexasHoldemHandCategory::FourOfAKind
        | TexasHoldemHandCategory::StraightFlush
        | TexasHoldemHandCategory::RoyalFlush => [kickers[0], 0, 0, 0, 0],
    }
}

impl PartialEq for EvaluatedHand {
    fn eq(&self, other: &Self) -> bool {
        (self.category_strength, self.kickers) == (other.category_strength, other.kickers)
    }
}

impl Eq for EvaluatedHand {}

impl PartialOrd for EvaluatedHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EvaluatedHand {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.category_strength, self.kickers).cmp(&(other.category_strength, other.kickers))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HandError {
    CardCount(usize),
    OmahaCardCount { hole: usize, community: usize },
    DuplicateCard(TexasHoldemCard),
    CardUnavailableInShortDeck(TexasHoldemCard),
}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CardCount(actual) => write!(f, "牌型判定需要 5..=7 张牌，实际为 {actual}"),
            Self::OmahaCardCount { hole, community } => write!(
                f,
                "奥马哈牌型判定需要 4 张底牌和 3..=5 张公共牌，实际为 {hole}+{community}"
            ),
            Self::DuplicateCard(card) => write!(f, "牌组中重复出现 {card}"),
            Self::CardUnavailableInShortDeck(card) => {
                write!(f, "短牌德州不能包含 {card}")
            }
        }
    }
}

impl std::error::Error for HandError {}

/// 从 5..=7 张底牌与公共牌中选择最大的五张牌。
pub fn evaluate_best(
    cards: &[TexasHoldemCard],
    rules: &TexasHoldemRuleSet,
) -> Result<EvaluatedHand, HandError> {
    if !(5..=7).contains(&cards.len()) {
        return Err(HandError::CardCount(cards.len()));
    }
    let mut seen = HashSet::with_capacity(cards.len());
    for card in cards {
        if !seen.insert(*card) {
            return Err(HandError::DuplicateCard(*card));
        }
        if rules.short_deck && card.rank().value() < TexasHoldemRank::Six.value() {
            return Err(HandError::CardUnavailableInShortDeck(*card));
        }
    }

    let mut best = None;
    for a in 0..cards.len() - 4 {
        for b in a + 1..cards.len() - 3 {
            for c in b + 1..cards.len() - 2 {
                for d in c + 1..cards.len() - 1 {
                    for e in d + 1..cards.len() {
                        let evaluated = evaluate_five(
                            [cards[a], cards[b], cards[c], cards[d], cards[e]],
                            rules.short_deck,
                        );
                        if best.is_none_or(|current| evaluated > current) {
                            best = Some(evaluated);
                        }
                    }
                }
            }
        }
    }
    Ok(best.expect("at least one five-card combination exists"))
}

/// 按当前房间玩法评估一名玩家的牌。标准德州可从全部底牌与公共牌中任选五张；
/// 奥马哈必须恰好选择两张底牌和三张公共牌。
pub fn evaluate_player_hand(
    hole_cards: &[TexasHoldemCard],
    community: &[TexasHoldemCard],
    rules: &TexasHoldemRuleSet,
) -> Result<EvaluatedHand, HandError> {
    if rules.omaha {
        evaluate_omaha(hole_cards, community, rules)
    } else {
        let mut cards = Vec::with_capacity(hole_cards.len() + community.len());
        cards.extend_from_slice(hole_cards);
        cards.extend_from_slice(community);
        evaluate_best(&cards, rules)
    }
}

/// 评估奥马哈高牌，严格枚举 2 张底牌与 3 张公共牌的所有组合。
pub fn evaluate_omaha(
    hole_cards: &[TexasHoldemCard],
    community: &[TexasHoldemCard],
    rules: &TexasHoldemRuleSet,
) -> Result<EvaluatedHand, HandError> {
    if hole_cards.len() != 4 || !(3..=5).contains(&community.len()) {
        return Err(HandError::OmahaCardCount {
            hole: hole_cards.len(),
            community: community.len(),
        });
    }
    let mut seen = HashSet::with_capacity(hole_cards.len() + community.len());
    for card in hole_cards.iter().chain(community) {
        if !seen.insert(*card) {
            return Err(HandError::DuplicateCard(*card));
        }
        if rules.short_deck && card.rank().value() < TexasHoldemRank::Six.value() {
            return Err(HandError::CardUnavailableInShortDeck(*card));
        }
    }

    let mut best = None;
    for first_hole in 0..hole_cards.len() - 1 {
        for second_hole in first_hole + 1..hole_cards.len() {
            for first_board in 0..community.len() - 2 {
                for second_board in first_board + 1..community.len() - 1 {
                    for third_board in second_board + 1..community.len() {
                        let evaluated = evaluate_five(
                            [
                                hole_cards[first_hole],
                                hole_cards[second_hole],
                                community[first_board],
                                community[second_board],
                                community[third_board],
                            ],
                            rules.short_deck,
                        );
                        if best.is_none_or(|current| evaluated > current) {
                            best = Some(evaluated);
                        }
                    }
                }
            }
        }
    }
    Ok(best.expect("four hole cards and at least three board cards form a combination"))
}

fn evaluate_five(cards: [TexasHoldemCard; 5], short_deck: bool) -> EvaluatedHand {
    let mut counts = [0_u8; 15];
    for card in cards {
        counts[usize::from(card.rank().value())] += 1;
    }
    let mut ranks = (2_u8..=14)
        .rev()
        .filter(|rank| counts[usize::from(*rank)] > 0)
        .collect::<Vec<_>>();
    let flush = cards.iter().all(|card| card.suit() == cards[0].suit());
    let straight = straight_high(&ranks, short_deck);
    let mut groups = (2_u8..=14)
        .filter_map(|rank| {
            let count = counts[usize::from(rank)];
            (count > 0).then_some((count, rank))
        })
        .collect::<Vec<_>>();
    groups.sort_unstable_by(|left, right| right.cmp(left));

    let (category, kickers) = if flush && straight == Some(14) && ranks == [14, 13, 12, 11, 10] {
        (TexasHoldemHandCategory::RoyalFlush, [14, 0, 0, 0, 0])
    } else if flush && straight.is_some() {
        (
            TexasHoldemHandCategory::StraightFlush,
            [straight.expect("checked"), 0, 0, 0, 0],
        )
    } else if groups[0].0 == 4 {
        (
            TexasHoldemHandCategory::FourOfAKind,
            [groups[0].1, groups[1].1, 0, 0, 0],
        )
    } else if groups[0].0 == 3 && groups[1].0 == 2 {
        (
            TexasHoldemHandCategory::FullHouse,
            [groups[0].1, groups[1].1, 0, 0, 0],
        )
    } else if flush {
        ranks.resize(5, 0);
        (
            TexasHoldemHandCategory::Flush,
            ranks.try_into().expect("five ranks"),
        )
    } else if let Some(high) = straight {
        (TexasHoldemHandCategory::Straight, [high, 0, 0, 0, 0])
    } else if groups[0].0 == 3 {
        let kickers = groups
            .iter()
            .skip(1)
            .map(|group| group.1)
            .collect::<Vec<_>>();
        (
            TexasHoldemHandCategory::ThreeOfAKind,
            [groups[0].1, kickers[0], kickers[1], 0, 0],
        )
    } else if groups[0].0 == 2 && groups[1].0 == 2 {
        let high_pair = groups[0].1.max(groups[1].1);
        let low_pair = groups[0].1.min(groups[1].1);
        (
            TexasHoldemHandCategory::TwoPair,
            [high_pair, low_pair, groups[2].1, 0, 0],
        )
    } else if groups[0].0 == 2 {
        let kickers = groups
            .iter()
            .skip(1)
            .map(|group| group.1)
            .collect::<Vec<_>>();
        (
            TexasHoldemHandCategory::OnePair,
            [groups[0].1, kickers[0], kickers[1], kickers[2], 0],
        )
    } else {
        ranks.resize(5, 0);
        (
            TexasHoldemHandCategory::HighCard,
            ranks.try_into().expect("five ranks"),
        )
    };

    EvaluatedHand {
        category,
        category_strength: category_strength(category, short_deck),
        kickers,
        cards: order_cards_for_display(cards, category, straight),
    }
}

/// Give the selected five cards a stable, poker-readable display order.
///
/// Made hands put their meaningful groups first (quads, trips, then pairs),
/// with equal-sized groups and kickers descending by rank. Straights are simply
/// descending, except that the ace is displayed last in A2345 and short-deck
/// A6789 because it acts as the lowest card there.
fn order_cards_for_display(
    mut cards: [TexasHoldemCard; 5],
    category: TexasHoldemHandCategory,
    straight_high: Option<u8>,
) -> [TexasHoldemCard; 5] {
    let mut counts = [0_u8; 15];
    for card in cards {
        counts[usize::from(card.rank().value())] += 1;
    }
    let low_ace_straight = matches!(
        category,
        TexasHoldemHandCategory::Straight | TexasHoldemHandCategory::StraightFlush
    ) && straight_high
        .is_some_and(|high| high < TexasHoldemRank::Ace.value());
    let display_rank = |card: TexasHoldemCard| {
        if low_ace_straight && card.rank() == TexasHoldemRank::Ace {
            1
        } else {
            card.rank().value()
        }
    };
    cards.sort_unstable_by(|left, right| {
        counts[usize::from(right.rank().value())]
            .cmp(&counts[usize::from(left.rank().value())])
            .then_with(|| display_rank(*right).cmp(&display_rank(*left)))
            .then_with(|| right.suit().cmp(&left.suit()))
    });
    cards
}

fn straight_high(ranks_descending: &[u8], short_deck: bool) -> Option<u8> {
    if ranks_descending.len() != 5 {
        return None;
    }
    if ranks_descending
        .windows(2)
        .all(|pair| pair[0] == pair[1] + 1)
    {
        return Some(ranks_descending[0]);
    }
    if !short_deck && ranks_descending == [14, 5, 4, 3, 2] {
        return Some(5);
    }
    if short_deck && ranks_descending == [14, 9, 8, 7, 6] {
        return Some(9);
    }
    None
}

const fn category_strength(category: TexasHoldemHandCategory, short_deck: bool) -> u8 {
    match (short_deck, category) {
        (_, TexasHoldemHandCategory::RoyalFlush) => 9,
        (_, TexasHoldemHandCategory::StraightFlush) => 8,
        (_, TexasHoldemHandCategory::FourOfAKind) => 7,
        (false, TexasHoldemHandCategory::FullHouse) | (true, TexasHoldemHandCategory::Flush) => 6,
        (false, TexasHoldemHandCategory::Flush) | (true, TexasHoldemHandCategory::FullHouse) => 5,
        (false, TexasHoldemHandCategory::Straight)
        | (true, TexasHoldemHandCategory::ThreeOfAKind) => 4,
        (false, TexasHoldemHandCategory::ThreeOfAKind)
        | (true, TexasHoldemHandCategory::Straight) => 3,
        (_, TexasHoldemHandCategory::TwoPair) => 2,
        (_, TexasHoldemHandCategory::OnePair) => 1,
        (_, TexasHoldemHandCategory::HighCard) => 0,
    }
}
