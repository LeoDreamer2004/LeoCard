use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::{BidTrump, Card, PlayerId, Rank, Suit, Trump};

/// 带王亮时，红色花色配大王，黑色花色配小王。
pub const fn bid_joker_for_suit(suit: Suit) -> Rank {
    match suit {
        Suit::Diamond | Suit::Heart => Rank::BigJoker,
        Suit::Club | Suit::Spade => Rank::SmallJoker,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BidKind {
    Initial,
    Protect,
    Counter,
    SelfCounter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Declaration {
    pub player: PlayerId,
    pub trump: BidTrump,
    pub kind: BidKind,
    pub protected: bool,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BidState {
    level: Rank,
    deck_count: u8,
    bid_with_joker: bool,
    original_bidder: Option<PlayerId>,
    current: Option<Declaration>,
    exposed: HashMap<Card, PlayerId>,
    closed: bool,
}

impl BidState {
    pub fn new(level: Rank) -> Self {
        Self::new_with_decks(level, 2)
    }

    pub fn new_with_decks(level: Rank, deck_count: u8) -> Self {
        Self::new_with_rules(level, deck_count, false)
    }

    pub fn new_with_rules(level: Rank, deck_count: u8, bid_with_joker: bool) -> Self {
        assert!(level.is_level_rank());
        assert!((2..=4).contains(&deck_count));
        Self {
            level,
            deck_count,
            bid_with_joker,
            original_bidder: None,
            current: None,
            exposed: HashMap::new(),
            closed: false,
        }
    }

    pub const fn level(&self) -> Rank {
        self.level
    }

    pub const fn original_bidder(&self) -> Option<PlayerId> {
        self.original_bidder
    }

    pub const fn current(&self) -> Option<&Declaration> {
        self.current.as_ref()
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// 返回指定玩家已经公开过的物理牌。客户端用它避免再次提交不能复用的级牌；
    /// 带王亮规则下的王仍可由同一玩家再次提交。
    pub fn exposed_cards(&self, player: PlayerId) -> Vec<Card> {
        let mut cards = self
            .exposed
            .iter()
            .filter_map(|(card, owner)| (*owner == player).then_some(*card))
            .collect::<Vec<_>>();
        cards.sort_by(Card::identity_cmp);
        cards
    }

    pub fn declare(
        &mut self,
        player: PlayerId,
        cards: &[Card],
        hand: &[Card],
    ) -> Result<&Declaration, BidError> {
        if self.closed {
            return Err(BidError::Closed);
        }
        if player.0 >= 4 {
            return Err(BidError::InvalidPlayer(player));
        }
        validate_owned(cards, hand)?;

        let next = match self.current.as_ref() {
            None => self.initial_declaration(player, cards)?,
            Some(current) => self.next_declaration(player, cards, current)?,
        };
        validate_reexposure(player, cards, &self.exposed, self.bid_with_joker)?;
        if self.original_bidder.is_none() {
            self.original_bidder = Some(player);
        }
        for card in cards {
            self.exposed.entry(*card).or_insert(player);
        }
        self.current = Some(next);
        Ok(self.current.as_ref().unwrap())
    }

    pub fn close(&mut self) -> Result<Trump, BidError> {
        self.closed = true;
        let declaration = self.current.as_ref().ok_or(BidError::NoDeclaration)?;
        Ok(Trump::new(self.level, declaration.trump.trump_suit()).unwrap())
    }

    /// 亮主窗口关闭后，抄底仍按普通反主强度校验，但不重新开放公开亮主阶段。
    pub fn counter_after_close(
        &mut self,
        player: PlayerId,
        cards: &[Card],
        hand: &[Card],
    ) -> Result<&Declaration, BidError> {
        if !self.closed {
            return Err(BidError::Closed);
        }
        validate_owned(cards, hand)?;
        let current = self.current.as_ref().ok_or(BidError::NoDeclaration)?;
        let next = self.next_declaration(player, cards, current)?;
        validate_reexposure(player, cards, &self.exposed, self.bid_with_joker)?;
        for card in cards {
            self.exposed.entry(*card).or_insert(player);
        }
        self.current = Some(next);
        Ok(self.current.as_ref().unwrap())
    }

    /// 返回该玩家当前能够用于反主抄底的候选牌。每种主牌目标只需保留一个
    /// 最小可行候选，客户端可据此绘制与亮主阶段相同的五个按钮。
    pub fn counter_options(&self, player: PlayerId, hand: &[Card]) -> Vec<Vec<Card>> {
        if !self.closed
            || self
                .current
                .as_ref()
                .is_none_or(|current| current.player == player)
        {
            return Vec::new();
        }
        let mut candidates = Vec::new();
        for suit in Suit::ALL {
            let level_cards = hand
                .iter()
                .copied()
                .filter(|card| card.rank() == self.level && card.suit() == Some(suit))
                .collect::<Vec<_>>();
            if self.bid_with_joker {
                let joker_rank = bid_joker_for_suit(suit);
                for joker in hand
                    .iter()
                    .copied()
                    .filter(|card| card.rank() == joker_rank && card.suit().is_none())
                {
                    for count in 1..=level_cards.len() {
                        let mut cards = level_cards[..count].to_vec();
                        cards.push(joker);
                        candidates.push(cards);
                    }
                }
            } else {
                for count in 1..=level_cards.len() {
                    candidates.push(level_cards[..count].to_vec());
                }
            }
        }
        for rank in [Rank::SmallJoker, Rank::BigJoker] {
            let jokers = hand
                .iter()
                .copied()
                .filter(|card| card.rank() == rank && card.suit().is_none())
                .collect::<Vec<_>>();
            for count in 2..=jokers.len() {
                candidates.push(jokers[..count].to_vec());
            }
        }
        candidates.retain(|cards| {
            validate_owned(cards, hand).is_ok()
                && self
                    .current
                    .as_ref()
                    .and_then(|current| self.next_declaration(player, cards, current).ok())
                    .is_some()
                && validate_reexposure(player, cards, &self.exposed, self.bid_with_joker).is_ok()
        });
        candidates.sort_by_key(Vec::len);
        candidates
    }

    fn initial_declaration(
        &self,
        player: PlayerId,
        cards: &[Card],
    ) -> Result<Declaration, BidError> {
        let parsed = parse_bid(cards, self.level, self.deck_count, self.bid_with_joker)?;
        if self.bid_with_joker && !matches!(parsed.trump, BidTrump::Suit(_)) {
            return Err(BidError::NoTrumpCannotOpen);
        }
        Ok(Declaration {
            player,
            trump: parsed.trump,
            kind: BidKind::Initial,
            // 保持两副牌原有的“直接亮一对即自保”语义。三、四副牌首次
            // 抢亮的同张数仍需参与无主 > 黑红梅方的同级比较。
            protected: self.deck_count == 2
                && parsed.primary_count == 2
                && matches!(parsed.trump, BidTrump::Suit(_)),
            cards: cards.to_vec(),
        })
    }

    fn next_declaration(
        &self,
        player: PlayerId,
        cards: &[Card],
        current: &Declaration,
    ) -> Result<Declaration, BidError> {
        let candidate_cards =
            if declaration_extends_current(player, cards, current, self.level, self.bid_with_joker)
            {
                merge_declaration_cards(&current.cards, cards)
            } else {
                cards.to_vec()
            };
        let parsed = parse_bid(
            &candidate_cards,
            self.level,
            self.deck_count,
            self.bid_with_joker,
        )?;
        let current_parsed = parse_bid(
            &current.cards,
            self.level,
            self.deck_count,
            self.bid_with_joker,
        )?;

        let protects = current.player == player
            && parsed.trump == current.trump
            && parsed.primary_count > current_parsed.primary_count
            && matches!(parsed.trump, BidTrump::Suit(_));
        if protects {
            return Ok(Declaration {
                player,
                trump: parsed.trump,
                kind: BidKind::Protect,
                protected: true,
                cards: candidate_cards,
            });
        }

        if parsed.primary_count < 2 {
            return Err(BidError::CounterRequiresPair);
        }
        // 自保后忽略花色强弱：同张数的其它花色不能反；无主可以反，更多
        // 级牌也可以反。后一种由张数优先的声明强度继续判定。
        if current.protected
            && matches!(parsed.trump, BidTrump::Suit(_))
            && parsed.primary_count <= current_parsed.primary_count
        {
            return Err(BidError::ProtectedSuit);
        }
        let candidate_strength = parsed
            .trump
            .declaration_strength(self.deck_count, parsed.primary_count)
            .ok_or(BidError::InvalidCards)?;
        let current_strength = current_parsed
            .trump
            .declaration_strength(self.deck_count, current_parsed.primary_count)
            .ok_or(BidError::InvalidCards)?;
        if candidate_strength <= current_strength {
            return Err(BidError::NotStronger);
        }
        Ok(Declaration {
            player,
            trump: parsed.trump,
            kind: if Some(player) == self.original_bidder {
                BidKind::SelfCounter
            } else {
                BidKind::Counter
            },
            protected: false,
            cards: candidate_cards,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParsedBid {
    trump: BidTrump,
    /// 声明强度只看同花色级牌或同种王的数量，不计带王亮附加的那张王。
    primary_count: usize,
}

fn parse_bid(
    cards: &[Card],
    level: Rank,
    deck_count: u8,
    bid_with_joker: bool,
) -> Result<ParsedBid, BidError> {
    if cards.is_empty() {
        return Err(BidError::InvalidCards);
    }

    if cards[1..].iter().all(|card| card.same_face(cards[0])) {
        let trump = match (cards[0].rank(), cards[0].suit()) {
            (rank, Some(suit)) if rank == level && !bid_with_joker => Some(BidTrump::Suit(suit)),
            (Rank::SmallJoker, None) if cards.len() >= 2 => Some(BidTrump::NoTrumpSmallJoker),
            (Rank::BigJoker, None) if cards.len() >= 2 => Some(BidTrump::NoTrumpBigJoker),
            _ => None,
        };
        if let Some(trump) = trump {
            let parsed = ParsedBid {
                trump,
                primary_count: cards.len(),
            };
            parsed
                .trump
                .declaration_strength(deck_count, parsed.primary_count)
                .ok_or(BidError::InvalidCards)?;
            return Ok(parsed);
        }
    }

    if bid_with_joker {
        for suit in Suit::ALL {
            let primary_count = cards
                .iter()
                .filter(|card| card.rank() == level && card.suit() == Some(suit))
                .count();
            let joker_rank = bid_joker_for_suit(suit);
            let joker_count = cards
                .iter()
                .filter(|card| card.rank() == joker_rank && card.suit().is_none())
                .count();
            if primary_count >= 1 && joker_count == 1 && primary_count + joker_count == cards.len()
            {
                let parsed = ParsedBid {
                    trump: BidTrump::Suit(suit),
                    primary_count,
                };
                parsed
                    .trump
                    .declaration_strength(deck_count, parsed.primary_count)
                    .ok_or(BidError::InvalidCards)?;
                return Ok(parsed);
            }
        }
        return Err(BidError::JokerRequired);
    }
    Err(BidError::InvalidCards)
}

fn declaration_extends_current(
    player: PlayerId,
    cards: &[Card],
    current: &Declaration,
    level: Rank,
    bid_with_joker: bool,
) -> bool {
    if current.player != player || cards.is_empty() {
        return false;
    }
    match current.trump {
        BidTrump::Suit(suit) => {
            let joker_rank = bid_joker_for_suit(suit);
            cards.iter().all(|card| {
                (card.rank() == level && card.suit() == Some(suit))
                    || (bid_with_joker
                        && card.rank() == joker_rank
                        && card.suit().is_none()
                        && current.cards.contains(card))
            }) && cards
                .iter()
                .any(|card| card.rank() == level && card.suit() == Some(suit))
        }
        BidTrump::NoTrumpSmallJoker => cards
            .iter()
            .all(|card| card.rank() == Rank::SmallJoker && card.suit().is_none()),
        BidTrump::NoTrumpBigJoker => cards
            .iter()
            .all(|card| card.rank() == Rank::BigJoker && card.suit().is_none()),
    }
}

fn merge_declaration_cards(current: &[Card], added: &[Card]) -> Vec<Card> {
    let mut combined = current.to_vec();
    for card in added {
        if !combined.contains(card) {
            combined.push(*card);
        }
    }
    combined
}

fn validate_owned(cards: &[Card], hand: &[Card]) -> Result<(), BidError> {
    if cards.is_empty() {
        return Err(BidError::InvalidCards);
    }
    let unique = cards.iter().copied().collect::<HashSet<_>>();
    if unique.len() != cards.len() || cards.iter().any(|card| !hand.contains(card)) {
        return Err(BidError::CardsNotOwned);
    }
    Ok(())
}

fn validate_reexposure(
    player: PlayerId,
    cards: &[Card],
    exposed: &HashMap<Card, PlayerId>,
    bid_with_joker: bool,
) -> Result<(), BidError> {
    if cards.iter().any(|card| {
        exposed.get(card).is_some_and(|owner| {
            *owner != player
                || !bid_with_joker
                || !matches!(card.rank(), Rank::SmallJoker | Rank::BigJoker)
        })
    }) {
        return Err(BidError::InvalidCards);
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BidError {
    InvalidPlayer(PlayerId),
    Closed,
    NoDeclaration,
    InvalidCards,
    CardsNotOwned,
    CounterRequiresPair,
    NotStronger,
    ProtectedSuit,
    JokerRequired,
    NoTrumpCannotOpen,
}

impl fmt::Display for BidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::Closed => f.write_str("亮主已经结束"),
            Self::NoDeclaration => f.write_str("无人亮主，本局必须重新发牌"),
            Self::InvalidCards => f.write_str("这些牌不能用于亮主或反主"),
            Self::CardsNotOwned => f.write_str("亮出的牌不全在玩家手中"),
            Self::CounterRequiresPair => f.write_str("单张不能互反，反主至少必须亮一对"),
            Self::NotStronger => f.write_str("只能用更高级的花色或无主反主"),
            Self::ProtectedSuit => f.write_str("自保后只能用无主或更多级牌反主"),
            Self::JokerRequired => f.write_str("带王亮必须同时亮出对应颜色的一张王"),
            Self::NoTrumpCannotOpen => f.write_str("带王亮规则下无主只能用于反主"),
        }
    }
}

impl std::error::Error for BidError {}

#[cfg(test)]
mod tests {
    use crate::Suit;

    use super::*;

    fn pair(suit: crate::Suit, rank: Rank) -> [Card; 2] {
        [Card::suited(0, suit, rank), Card::suited(1, suit, rank)]
    }

    #[test]
    fn suit_counters_follow_diamond_club_heart_spade_order() {
        let diamond = pair(Suit::Diamond, Rank::Ten);
        let heart = pair(Suit::Heart, Rank::Ten);
        let club = pair(Suit::Club, Rank::Ten);
        let hand = [diamond, heart, club].concat();
        let mut bidding = BidState::new(Rank::Ten);
        bidding.declare(PlayerId(0), &diamond[..1], &hand).unwrap();
        bidding.declare(PlayerId(1), &heart, &hand).unwrap();
        assert_eq!(
            bidding.declare(PlayerId(2), &club, &hand),
            Err(BidError::NotStronger)
        );
    }

    #[test]
    fn self_protection_only_allows_no_trump_and_big_jokers_beat_small() {
        let diamond = pair(Suit::Diamond, Rank::Ten);
        let spade = pair(Suit::Spade, Rank::Ten);
        let small = [Card::small_joker(0), Card::small_joker(1)];
        let big = [Card::big_joker(0), Card::big_joker(1)];
        let hand = [diamond.as_slice(), spade.as_slice(), &small, &big].concat();
        let mut bidding = BidState::new(Rank::Ten);
        bidding.declare(PlayerId(0), &diamond[..1], &hand).unwrap();
        bidding.declare(PlayerId(0), &diamond[1..], &hand).unwrap();
        assert_eq!(
            bidding.declare(PlayerId(1), &spade, &hand),
            Err(BidError::ProtectedSuit)
        );
        bidding.declare(PlayerId(1), &small, &hand).unwrap();
        bidding.declare(PlayerId(2), &big, &hand).unwrap();
        assert_eq!(bidding.current().unwrap().trump, BidTrump::NoTrumpBigJoker);
    }

    #[test]
    fn original_bidder_can_self_counter_with_another_suit_pair() {
        let diamond = pair(Suit::Diamond, Rank::Ten);
        let heart = pair(Suit::Heart, Rank::Ten);
        let spade = pair(Suit::Spade, Rank::Ten);
        let hand = [diamond, heart, spade].concat();
        let mut bidding = BidState::new(Rank::Ten);
        bidding.declare(PlayerId(0), &diamond[..1], &hand).unwrap();
        bidding.declare(PlayerId(1), &heart, &hand).unwrap();
        let declaration = bidding.declare(PlayerId(0), &spade, &hand).unwrap();
        assert_eq!(declaration.kind, BidKind::SelfCounter);
    }

    #[test]
    fn three_deck_single_bids_cannot_counter_and_reinforcement_uses_more_copies() {
        let diamond = [
            Card::suited(0, Suit::Diamond, Rank::Ten),
            Card::suited(1, Suit::Diamond, Rank::Ten),
            Card::suited(2, Suit::Diamond, Rank::Ten),
        ];
        let heart_single = Card::suited(0, Suit::Heart, Rank::Ten);
        let hand = [diamond.as_slice(), &[heart_single]].concat();
        let mut bidding = BidState::new_with_decks(Rank::Ten, 3);

        bidding.declare(PlayerId(0), &diamond[..1], &hand).unwrap();
        assert_eq!(
            bidding.declare(PlayerId(1), &[heart_single], &hand),
            Err(BidError::CounterRequiresPair)
        );
        let pair = bidding.declare(PlayerId(0), &diamond[1..2], &hand).unwrap();
        assert_eq!(pair.cards, diamond[..2]);
        let triple = bidding.declare(PlayerId(0), &diamond[2..], &hand).unwrap();
        assert_eq!(triple.cards, diamond);
    }

    #[test]
    fn three_deck_bid_strength_uses_count_then_no_trump_and_suit_order() {
        let diamond_pair = BidTrump::Suit(Suit::Diamond)
            .three_deck_declaration_strength(2)
            .unwrap();
        let spade_pair = BidTrump::Suit(Suit::Spade)
            .three_deck_declaration_strength(2)
            .unwrap();
        let small_pair = BidTrump::NoTrumpSmallJoker
            .three_deck_declaration_strength(2)
            .unwrap();
        let big_pair = BidTrump::NoTrumpBigJoker
            .three_deck_declaration_strength(2)
            .unwrap();
        let diamond_triple = BidTrump::Suit(Suit::Diamond)
            .three_deck_declaration_strength(3)
            .unwrap();
        assert!(diamond_pair < spade_pair);
        assert!(spade_pair < small_pair);
        assert!(small_pair < big_pair);
        assert!(big_pair < diamond_triple);
    }

    #[test]
    fn four_deck_bidding_extends_to_quad_level_and_quad_jokers() {
        let diamond = (0..4)
            .map(|deck| Card::suited(deck, Suit::Diamond, Rank::Ten))
            .collect::<Vec<_>>();
        let spade = (0..4)
            .map(|deck| Card::suited(deck, Suit::Spade, Rank::Ten))
            .collect::<Vec<_>>();
        let big = (0..4).map(Card::big_joker).collect::<Vec<_>>();
        let hand = [diamond.as_slice(), spade.as_slice(), big.as_slice()].concat();
        let mut bidding = BidState::new_with_decks(Rank::Ten, 4);

        bidding.declare(PlayerId(0), &diamond[..1], &hand).unwrap();
        assert_eq!(
            bidding.declare(PlayerId(1), &spade[..1], &hand),
            Err(BidError::CounterRequiresPair)
        );
        bidding.declare(PlayerId(0), &diamond[1..2], &hand).unwrap();
        bidding.declare(PlayerId(0), &diamond[2..3], &hand).unwrap();
        bidding.declare(PlayerId(0), &diamond[3..], &hand).unwrap();
        let declaration = bidding.declare(PlayerId(2), &big, &hand).unwrap();
        assert_eq!(declaration.trump, BidTrump::NoTrumpBigJoker);
        assert_eq!(declaration.cards.len(), 4);
    }

    #[test]
    fn four_deck_strength_orders_count_before_no_trump_and_suit() {
        let big_triple = BidTrump::NoTrumpBigJoker
            .declaration_strength(4, 3)
            .unwrap();
        let diamond_quad = BidTrump::Suit(Suit::Diamond)
            .declaration_strength(4, 4)
            .unwrap();
        let spade_quad = BidTrump::Suit(Suit::Spade)
            .declaration_strength(4, 4)
            .unwrap();
        let small_quad = BidTrump::NoTrumpSmallJoker
            .declaration_strength(4, 4)
            .unwrap();
        let big_quad = BidTrump::NoTrumpBigJoker
            .declaration_strength(4, 4)
            .unwrap();
        assert!(big_triple < diamond_quad);
        assert!(diamond_quad < spade_quad);
        assert!(spade_quad < small_quad);
        assert!(small_quad < big_quad);
    }

    #[test]
    fn joker_bidding_requires_the_matching_color_and_cannot_open_no_trump() {
        let heart = Card::suited(0, Suit::Heart, Rank::Ten);
        let small = Card::small_joker(0);
        let big = [Card::big_joker(0), Card::big_joker(1)];
        let hand = [heart, small, big[0], big[1]];
        let mut bidding = BidState::new_with_rules(Rank::Ten, 2, true);

        assert_eq!(
            bidding.declare(PlayerId(0), &[heart], &hand),
            Err(BidError::JokerRequired)
        );
        assert_eq!(
            bidding.declare(PlayerId(0), &[heart, small], &hand),
            Err(BidError::JokerRequired)
        );
        assert_eq!(
            bidding.declare(PlayerId(0), &big, &hand),
            Err(BidError::NoTrumpCannotOpen)
        );
        let declaration = bidding
            .declare(PlayerId(0), &[heart, big[0]], &hand)
            .unwrap();
        assert_eq!(declaration.trump, BidTrump::Suit(Suit::Heart));
    }

    #[test]
    fn joker_bidding_reuses_the_owners_exposed_joker_for_protection_and_no_trump() {
        let heart = pair(Suit::Heart, Rank::Ten);
        let big = [Card::big_joker(0), Card::big_joker(1)];
        let hand = [heart.as_slice(), big.as_slice()].concat();
        let mut bidding = BidState::new_with_rules(Rank::Ten, 2, true);

        bidding
            .declare(PlayerId(0), &[heart[0], big[0]], &hand)
            .unwrap();
        assert_eq!(
            bidding.declare(PlayerId(1), &big, &hand),
            Err(BidError::InvalidCards),
            "其它玩家不能借用已经亮出的王"
        );
        let protected = bidding.declare(PlayerId(0), &[heart[1]], &hand).unwrap();
        assert!(protected.protected);
        assert_eq!(protected.kind, BidKind::Protect);
        assert_eq!(protected.cards, vec![heart[0], big[0], heart[1]]);

        let no_trump = bidding.declare(PlayerId(0), &big, &hand).unwrap();
        assert_eq!(no_trump.trump, BidTrump::NoTrumpBigJoker);
        assert_eq!(no_trump.cards, big);
    }

    #[test]
    fn protected_joker_bid_ignores_suit_order_but_more_level_cards_can_counter() {
        let diamond = (0..3)
            .map(|deck| Card::suited(deck, Suit::Diamond, Rank::Ten))
            .collect::<Vec<_>>();
        let spade = (0..3)
            .map(|deck| Card::suited(deck, Suit::Spade, Rank::Ten))
            .collect::<Vec<_>>();
        let big = [Card::big_joker(0), Card::big_joker(1)];
        let small = [Card::small_joker(0), Card::small_joker(1)];
        let hand = [
            diamond.as_slice(),
            spade.as_slice(),
            big.as_slice(),
            small.as_slice(),
        ]
        .concat();

        let mut same_count = BidState::new_with_rules(Rank::Ten, 3, true);
        same_count
            .declare(PlayerId(0), &[diamond[0], big[0]], &hand)
            .unwrap();
        same_count
            .declare(PlayerId(0), &[diamond[1]], &hand)
            .unwrap();
        assert_eq!(
            same_count.declare(PlayerId(1), &[spade[0], spade[1], small[0]], &hand),
            Err(BidError::ProtectedSuit)
        );

        let mut more_cards = BidState::new_with_rules(Rank::Ten, 3, true);
        more_cards
            .declare(PlayerId(0), &[spade[0], small[0]], &hand)
            .unwrap();
        more_cards.declare(PlayerId(0), &[spade[1]], &hand).unwrap();
        let counter = more_cards
            .declare(
                PlayerId(1),
                &[diamond[0], diamond[1], diamond[2], big[1]],
                &hand,
            )
            .unwrap();
        assert_eq!(counter.trump, BidTrump::Suit(Suit::Diamond));
    }
}
