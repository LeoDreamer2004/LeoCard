use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

use crate::{Card, Rank, RuleSet};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HandCategory {
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
    category: HandCategory,
    category_strength: u8,
    kickers: [u8; 5],
    cards: [Card; 5],
}

impl EvaluatedHand {
    pub const fn category(self) -> HandCategory {
        self.category
    }

    pub const fn kickers(self) -> [u8; 5] {
        self.kickers
    }

    pub const fn cards(self) -> [Card; 5] {
        self.cards
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
    DuplicateCard(Card),
    CardUnavailableInShortDeck(Card),
}

impl fmt::Display for HandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CardCount(actual) => write!(f, "牌型判定需要 5..=7 张牌，实际为 {actual}"),
            Self::DuplicateCard(card) => write!(f, "牌组中重复出现 {card}"),
            Self::CardUnavailableInShortDeck(card) => {
                write!(f, "短牌德州不能包含 {card}")
            }
        }
    }
}

impl std::error::Error for HandError {}

/// 从 5..=7 张底牌与公共牌中选择最大的五张牌。
pub fn evaluate_best(cards: &[Card], rules: &RuleSet) -> Result<EvaluatedHand, HandError> {
    if !(5..=7).contains(&cards.len()) {
        return Err(HandError::CardCount(cards.len()));
    }
    let mut seen = HashSet::with_capacity(cards.len());
    for card in cards {
        if !seen.insert(*card) {
            return Err(HandError::DuplicateCard(*card));
        }
        if rules.short_deck && card.rank().value() < Rank::Six.value() {
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

fn evaluate_five(cards: [Card; 5], short_deck: bool) -> EvaluatedHand {
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
        (HandCategory::RoyalFlush, [14, 0, 0, 0, 0])
    } else if flush && straight.is_some() {
        (
            HandCategory::StraightFlush,
            [straight.expect("checked"), 0, 0, 0, 0],
        )
    } else if groups[0].0 == 4 {
        (
            HandCategory::FourOfAKind,
            [groups[0].1, groups[1].1, 0, 0, 0],
        )
    } else if groups[0].0 == 3 && groups[1].0 == 2 {
        (HandCategory::FullHouse, [groups[0].1, groups[1].1, 0, 0, 0])
    } else if flush {
        ranks.resize(5, 0);
        (HandCategory::Flush, ranks.try_into().expect("five ranks"))
    } else if let Some(high) = straight {
        (HandCategory::Straight, [high, 0, 0, 0, 0])
    } else if groups[0].0 == 3 {
        let kickers = groups
            .iter()
            .skip(1)
            .map(|group| group.1)
            .collect::<Vec<_>>();
        (
            HandCategory::ThreeOfAKind,
            [groups[0].1, kickers[0], kickers[1], 0, 0],
        )
    } else if groups[0].0 == 2 && groups[1].0 == 2 {
        let high_pair = groups[0].1.max(groups[1].1);
        let low_pair = groups[0].1.min(groups[1].1);
        (
            HandCategory::TwoPair,
            [high_pair, low_pair, groups[2].1, 0, 0],
        )
    } else if groups[0].0 == 2 {
        let kickers = groups
            .iter()
            .skip(1)
            .map(|group| group.1)
            .collect::<Vec<_>>();
        (
            HandCategory::OnePair,
            [groups[0].1, kickers[0], kickers[1], kickers[2], 0],
        )
    } else {
        ranks.resize(5, 0);
        (
            HandCategory::HighCard,
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
    mut cards: [Card; 5],
    category: HandCategory,
    straight_high: Option<u8>,
) -> [Card; 5] {
    let mut counts = [0_u8; 15];
    for card in cards {
        counts[usize::from(card.rank().value())] += 1;
    }
    let low_ace_straight = matches!(
        category,
        HandCategory::Straight | HandCategory::StraightFlush
    ) && straight_high.is_some_and(|high| high < Rank::Ace.value());
    let display_rank = |card: Card| {
        if low_ace_straight && card.rank() == Rank::Ace {
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

const fn category_strength(category: HandCategory, short_deck: bool) -> u8 {
    match (short_deck, category) {
        (_, HandCategory::RoyalFlush) => 9,
        (_, HandCategory::StraightFlush) => 8,
        (_, HandCategory::FourOfAKind) => 7,
        (false, HandCategory::FullHouse) | (true, HandCategory::Flush) => 6,
        (false, HandCategory::Flush) | (true, HandCategory::FullHouse) => 5,
        (false, HandCategory::Straight) | (true, HandCategory::ThreeOfAKind) => 4,
        (false, HandCategory::ThreeOfAKind) | (true, HandCategory::Straight) => 3,
        (_, HandCategory::TwoPair) => 2,
        (_, HandCategory::OnePair) => 1,
        (_, HandCategory::HighCard) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Suit::{Club, Diamond, Heart, Spade};

    fn c(rank: Rank, suit: crate::Suit) -> Card {
        Card::new(suit, rank)
    }

    fn rules(short_deck: bool) -> RuleSet {
        RuleSet {
            short_deck,
            ..RuleSet::default()
        }
    }

    #[test]
    fn standard_category_order_matches_holdem() {
        let flush = evaluate_best(
            &[
                c(Rank::Ace, Heart),
                c(Rank::Jack, Heart),
                c(Rank::Nine, Heart),
                c(Rank::Seven, Heart),
                c(Rank::Three, Heart),
            ],
            &rules(false),
        )
        .unwrap();
        let full_house = evaluate_best(
            &[
                c(Rank::King, Spade),
                c(Rank::King, Heart),
                c(Rank::King, Club),
                c(Rank::Nine, Spade),
                c(Rank::Nine, Diamond),
            ],
            &rules(false),
        )
        .unwrap();
        let straight = evaluate_best(
            &[
                c(Rank::Nine, Spade),
                c(Rank::Eight, Heart),
                c(Rank::Seven, Club),
                c(Rank::Six, Diamond),
                c(Rank::Five, Heart),
            ],
            &rules(false),
        )
        .unwrap();
        let trips = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::Ace, Club),
                c(Rank::King, Diamond),
                c(Rank::Queen, Heart),
            ],
            &rules(false),
        )
        .unwrap();
        assert!(full_house > flush);
        assert!(flush > straight);
        assert!(straight > trips);
    }

    #[test]
    fn short_deck_moves_flush_and_trips_above_full_house_and_straight() {
        let short = rules(true);
        let flush = evaluate_best(
            &[
                c(Rank::Ace, Heart),
                c(Rank::Jack, Heart),
                c(Rank::Nine, Heart),
                c(Rank::Seven, Heart),
                c(Rank::Six, Heart),
            ],
            &short,
        )
        .unwrap();
        let full_house = evaluate_best(
            &[
                c(Rank::King, Spade),
                c(Rank::King, Heart),
                c(Rank::King, Club),
                c(Rank::Nine, Spade),
                c(Rank::Nine, Diamond),
            ],
            &short,
        )
        .unwrap();
        let trips = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::Ace, Club),
                c(Rank::King, Diamond),
                c(Rank::Queen, Heart),
            ],
            &short,
        )
        .unwrap();
        let straight = evaluate_best(
            &[
                c(Rank::Ten, Spade),
                c(Rank::Nine, Heart),
                c(Rank::Eight, Club),
                c(Rank::Seven, Diamond),
                c(Rank::Six, Heart),
            ],
            &short,
        )
        .unwrap();
        assert!(flush > full_house);
        assert!(trips > straight);
    }

    #[test]
    fn royal_flush_is_distinct_and_seven_cards_choose_the_best_five() {
        let hand = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::King, Spade),
                c(Rank::Queen, Spade),
                c(Rank::Jack, Spade),
                c(Rank::Ten, Spade),
                c(Rank::Two, Club),
                c(Rank::Two, Diamond),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(hand.category(), HandCategory::RoyalFlush);
    }

    #[test]
    fn ace_can_be_low_in_each_deck_variant() {
        let standard = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Five, Heart),
                c(Rank::Four, Club),
                c(Rank::Three, Diamond),
                c(Rank::Two, Heart),
            ],
            &rules(false),
        )
        .unwrap();
        let short = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Nine, Heart),
                c(Rank::Eight, Club),
                c(Rank::Seven, Diamond),
                c(Rank::Six, Heart),
            ],
            &rules(true),
        )
        .unwrap();
        assert_eq!(standard.kickers()[0], 5);
        assert_eq!(short.kickers()[0], 9);

        let standard_six_high = evaluate_best(
            &[
                c(Rank::Six, Spade),
                c(Rank::Five, Diamond),
                c(Rank::Four, Heart),
                c(Rank::Three, Club),
                c(Rank::Two, Spade),
            ],
            &rules(false),
        )
        .unwrap();
        let short_ten_high = evaluate_best(
            &[
                c(Rank::Ten, Spade),
                c(Rank::Nine, Diamond),
                c(Rank::Eight, Heart),
                c(Rank::Seven, Club),
                c(Rank::Six, Spade),
            ],
            &rules(true),
        )
        .unwrap();
        assert!(standard < standard_six_high, "A2345 必须是普通德州最小顺子");
        assert!(short < short_ten_high, "A6789 必须是短牌德州最小顺子");
    }

    #[test]
    fn selected_five_cards_have_poker_readable_display_order() {
        let quads = evaluate_best(
            &[
                c(Rank::Ace, Diamond),
                c(Rank::Nine, Club),
                c(Rank::Nine, Spade),
                c(Rank::Nine, Heart),
                c(Rank::Nine, Diamond),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(
            quads.cards().map(Card::rank),
            [Rank::Nine, Rank::Nine, Rank::Nine, Rank::Nine, Rank::Ace]
        );

        let full_house = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Two, Club),
                c(Rank::Ace, Heart),
                c(Rank::Two, Spade),
                c(Rank::Two, Diamond),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(
            full_house.cards().map(Card::rank),
            [Rank::Two, Rank::Two, Rank::Two, Rank::Ace, Rank::Ace]
        );

        let pair = evaluate_best(
            &[
                c(Rank::Queen, Diamond),
                c(Rank::Six, Spade),
                c(Rank::Ace, Heart),
                c(Rank::Six, Club),
                c(Rank::King, Diamond),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(
            pair.cards().map(Card::rank),
            [Rank::Six, Rank::Six, Rank::Ace, Rank::King, Rank::Queen]
        );

        let high_card = evaluate_best(
            &[
                c(Rank::Nine, Spade),
                c(Rank::Queen, Club),
                c(Rank::Ace, Diamond),
                c(Rank::Jack, Heart),
                c(Rank::King, Spade),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(
            high_card.cards().map(Card::rank),
            [Rank::Ace, Rank::King, Rank::Queen, Rank::Jack, Rank::Nine]
        );

        let wheel = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Three, Club),
                c(Rank::Five, Heart),
                c(Rank::Two, Diamond),
                c(Rank::Four, Spade),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(
            wheel.cards().map(Card::rank),
            [Rank::Five, Rank::Four, Rank::Three, Rank::Two, Rank::Ace]
        );

        let short_wheel = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Seven, Club),
                c(Rank::Nine, Heart),
                c(Rank::Six, Diamond),
                c(Rank::Eight, Spade),
            ],
            &rules(true),
        )
        .unwrap();
        assert_eq!(
            short_wheel.cards().map(Card::rank),
            [Rank::Nine, Rank::Eight, Rank::Seven, Rank::Six, Rank::Ace]
        );
    }

    #[test]
    fn every_declared_standard_category_is_in_exact_order() {
        let hands = [
            [
                c(Rank::Ace, Spade),
                c(Rank::King, Spade),
                c(Rank::Queen, Spade),
                c(Rank::Jack, Spade),
                c(Rank::Ten, Spade),
            ],
            [
                c(Rank::Nine, Heart),
                c(Rank::Eight, Heart),
                c(Rank::Seven, Heart),
                c(Rank::Six, Heart),
                c(Rank::Five, Heart),
            ],
            [
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::Ace, Club),
                c(Rank::Ace, Diamond),
                c(Rank::King, Spade),
            ],
            [
                c(Rank::King, Spade),
                c(Rank::King, Heart),
                c(Rank::King, Club),
                c(Rank::Queen, Spade),
                c(Rank::Queen, Heart),
            ],
            [
                c(Rank::Ace, Club),
                c(Rank::Jack, Club),
                c(Rank::Nine, Club),
                c(Rank::Seven, Club),
                c(Rank::Three, Club),
            ],
            [
                c(Rank::Ten, Spade),
                c(Rank::Nine, Heart),
                c(Rank::Eight, Club),
                c(Rank::Seven, Diamond),
                c(Rank::Six, Spade),
            ],
            [
                c(Rank::Jack, Spade),
                c(Rank::Jack, Heart),
                c(Rank::Jack, Club),
                c(Rank::Ace, Diamond),
                c(Rank::King, Spade),
            ],
            [
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::King, Club),
                c(Rank::King, Diamond),
                c(Rank::Queen, Spade),
            ],
            [
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::King, Club),
                c(Rank::Queen, Diamond),
                c(Rank::Jack, Spade),
            ],
            [
                c(Rank::Ace, Spade),
                c(Rank::King, Heart),
                c(Rank::Queen, Club),
                c(Rank::Jack, Diamond),
                c(Rank::Nine, Spade),
            ],
        ]
        .map(|cards| evaluate_best(&cards, &rules(false)).unwrap());
        assert!(hands.windows(2).all(|pair| pair[0] > pair[1]));
        assert_eq!(
            hands.map(EvaluatedHand::category),
            [
                HandCategory::RoyalFlush,
                HandCategory::StraightFlush,
                HandCategory::FourOfAKind,
                HandCategory::FullHouse,
                HandCategory::Flush,
                HandCategory::Straight,
                HandCategory::ThreeOfAKind,
                HandCategory::TwoPair,
                HandCategory::OnePair,
                HandCategory::HighCard,
            ]
        );
    }

    #[test]
    fn equal_rank_hands_split_regardless_of_suit() {
        let left = evaluate_best(
            &[
                c(Rank::Ace, Spade),
                c(Rank::Ace, Heart),
                c(Rank::King, Club),
                c(Rank::Queen, Diamond),
                c(Rank::Jack, Spade),
            ],
            &rules(false),
        )
        .unwrap();
        let right = evaluate_best(
            &[
                c(Rank::Ace, Club),
                c(Rank::Ace, Diamond),
                c(Rank::King, Heart),
                c(Rank::Queen, Spade),
                c(Rank::Jack, Club),
            ],
            &rules(false),
        )
        .unwrap();
        assert_eq!(left, right);
    }
}
