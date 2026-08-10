use std::fmt;

use crate::{Card, Rank, Suit};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerId(pub u8);

impl PlayerId {
    pub const fn team(self) -> TeamId {
        TeamId(self.0 % 2)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamId(pub u8);

impl TeamId {
    pub const fn other(self) -> Self {
        Self(1 - self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ThrowPenalty {
    None,
    FivePerCard,
    TenPerCard,
}

impl ThrowPenalty {
    pub const fn points_per_card(self) -> u16 {
        match self {
            Self::None => 0,
            Self::FivePerCard => 5,
            Self::TenPerCard => 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RuleSet {
    /// 使用两至四副牌；三副加入泰坦尼克，四副再加入炸弹和宇宙飞船。
    pub deck_count: u8,
    pub allow_throw: bool,
    pub throw_penalty: ThrowPenalty,
    pub mandatory_five_ten_king_ace: bool,
    /// 开启后有花色亮主必须同时带同色王，且无主只能用于反主。
    #[cfg_attr(feature = "serde", serde(default))]
    pub bid_with_joker: bool,
    /// 第一次无人亮主时不重新发牌，改由当前庄家的下家接庄并再亮十秒。
    #[cfg_attr(feature = "serde", serde(default))]
    pub power_outage_dealer: bool,
    /// 最终无人亮主时逐张翻开底牌，并按手牌中的同牌数量确定庄家和主牌。
    #[cfg_attr(feature = "serde", serde(default))]
    pub bottom_flip: bool,
    /// 庄家首次埋底后，允许尚未亮出的更强反主牌按座次依次抄底并重新埋底。
    #[cfg_attr(feature = "serde", serde(default))]
    pub bottom_copy: bool,
    /// 开启后有主局在埋底完成后允许主牌不多于五张的玩家与对家交换。
    #[cfg_attr(feature = "serde", serde(default))]
    pub five_trump_crossing: bool,
    /// 开启后四张 2 永远属于主牌，双方从 3 级开始。
    pub constant_trump: bool,
}

impl RuleSet {
    pub const PLAYER_COUNT: usize = 4;
    /// 两副牌模式的兼容常量；规则计算应优先使用 [`Self::kitty_size`]。
    pub const KITTY_SIZE: usize = 8;
    /// 两副牌模式的兼容常量；规则计算应优先使用 [`Self::hand_size`]。
    pub const HAND_SIZE: usize = 25;

    pub const fn validate(self) -> Result<Self, RuleError> {
        match self.deck_count {
            2..=4 => Ok(self),
            actual => Err(RuleError::InvalidDeckCount(actual)),
        }
    }

    pub const fn kitty_size(self) -> usize {
        if self.deck_count == 3 { 6 } else { 8 }
    }

    pub const fn hand_size(self) -> usize {
        (self.deck_count as usize * 54 - self.kitty_size()) / Self::PLAYER_COUNT
    }

    pub const fn score_step(self) -> u32 {
        match self.deck_count {
            4 => 80,
            3 => 60,
            _ => 40,
        }
    }

    pub const fn takeover_score(self) -> u32 {
        match self.deck_count {
            4 => 160,
            3 => 120,
            _ => 80,
        }
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            deck_count: 2,
            allow_throw: true,
            throw_penalty: ThrowPenalty::None,
            mandatory_five_ten_king_ace: false,
            bid_with_joker: false,
            power_outage_dealer: false,
            bottom_flip: false,
            bottom_copy: false,
            five_trump_crossing: false,
            constant_trump: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleError {
    InvalidDeckCount(u8),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDeckCount(actual) => {
                write!(f, "升级只支持两至四副牌，实际为 {actual} 副")
            }
        }
    }
}

impl std::error::Error for RuleError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BidTrump {
    Suit(Suit),
    NoTrumpSmallJoker,
    NoTrumpBigJoker,
}

impl BidTrump {
    pub const fn strength(self) -> u8 {
        match self {
            Self::Suit(suit) => suit.bid_strength(),
            Self::NoTrumpSmallJoker => 4,
            Self::NoTrumpBigJoker => 5,
        }
    }

    pub const fn trump_suit(self) -> Option<Suit> {
        match self {
            Self::Suit(suit) => Some(suit),
            Self::NoTrumpSmallJoker | Self::NoTrumpBigJoker => None,
        }
    }

    /// 三副牌抢亮的完整强度：先比较亮牌层级，再按无主、黑、红、梅、方比较。
    /// 大王和小王都属于无主，同张数时大王更高。
    pub const fn three_deck_declaration_strength(self, card_count: usize) -> Option<u8> {
        match (card_count, self) {
            (1, Self::Suit(suit)) => Some(suit.bid_strength()),
            (2, Self::Suit(suit)) => Some(4 + suit.bid_strength()),
            (2, Self::NoTrumpSmallJoker) => Some(8),
            (2, Self::NoTrumpBigJoker) => Some(9),
            (3, Self::Suit(suit)) => Some(10 + suit.bid_strength()),
            (3, Self::NoTrumpSmallJoker) => Some(14),
            (3, Self::NoTrumpBigJoker) => Some(15),
            _ => None,
        }
    }

    /// 四副牌在三副牌层级之上继续加入四张级牌和四张王。
    pub const fn four_deck_declaration_strength(self, card_count: usize) -> Option<u8> {
        match (card_count, self) {
            (1, Self::Suit(suit)) => Some(suit.bid_strength()),
            (2, Self::Suit(suit)) => Some(4 + suit.bid_strength()),
            (2, Self::NoTrumpSmallJoker) => Some(8),
            (2, Self::NoTrumpBigJoker) => Some(9),
            (3, Self::Suit(suit)) => Some(10 + suit.bid_strength()),
            (3, Self::NoTrumpSmallJoker) => Some(14),
            (3, Self::NoTrumpBigJoker) => Some(15),
            (4, Self::Suit(suit)) => Some(16 + suit.bid_strength()),
            (4, Self::NoTrumpSmallJoker) => Some(20),
            (4, Self::NoTrumpBigJoker) => Some(21),
            _ => None,
        }
    }

    pub const fn declaration_strength(self, deck_count: u8, card_count: usize) -> Option<u8> {
        if deck_count == 4 {
            self.four_deck_declaration_strength(card_count)
        } else {
            self.three_deck_declaration_strength(card_count)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trump {
    pub level: Rank,
    pub suit: Option<Suit>,
    pub constant_trump: bool,
}

impl Trump {
    pub fn new(level: Rank, suit: Option<Suit>) -> Result<Self, RuleError> {
        assert!(level.is_level_rank(), "jokers cannot be a level");
        Ok(Self {
            level,
            suit,
            constant_trump: false,
        })
    }

    pub const fn with_constant_trump(mut self, enabled: bool) -> Self {
        self.constant_trump = enabled;
        self
    }

    pub fn is_trump(self, card: Card) -> bool {
        matches!(card.rank(), Rank::SmallJoker | Rank::BigJoker)
            || card.rank() == self.level
            || (self.constant_trump && card.rank() == Rank::Two)
            || match (self.suit, card.suit()) {
                (Some(trump), Some(card_suit)) => trump as u8 == card_suit as u8,
                _ => false,
            }
    }

    /// 同门牌力。返回值同时是对子构成拖拉机时使用的相邻层级。
    pub fn strength(self, card: Card) -> u8 {
        let has_separate_constant = self.constant_trump && self.level != Rank::Two;
        if card.rank() == Rank::BigJoker {
            return match (self.suit.is_some(), has_separate_constant) {
                (true, true) => 16,
                (true, false) => 15,
                (false, true) => 3,
                (false, false) => 2,
            };
        }
        if card.rank() == Rank::SmallJoker {
            return match (self.suit.is_some(), has_separate_constant) {
                (true, true) => 15,
                (true, false) => 14,
                (false, true) => 2,
                (false, false) => 1,
            };
        }
        if card.rank() == self.level {
            return match self.suit {
                Some(suit) if card.suit() == Some(suit) => {
                    if has_separate_constant {
                        14
                    } else {
                        13
                    }
                }
                Some(_) => {
                    if has_separate_constant {
                        13
                    } else {
                        12
                    }
                }
                None => u8::from(has_separate_constant),
            };
        }
        if has_separate_constant && card.rank() == Rank::Two {
            return match self.suit {
                Some(suit) if card.suit() == Some(suit) => 12,
                Some(_) => 11,
                None => 0,
            };
        }

        let raw = card.rank().level_index().expect("suited card rank");
        let level = self.level.level_index().expect("validated level");
        raw - u8::from(raw > level) - u8::from(has_separate_constant && raw > 0)
    }
}

pub fn level_after(current: Rank, steps: u8, mandatory: bool) -> Rank {
    let start = current.level_index().expect("level rank") as usize;
    let target = (start + usize::from(steps)).min(Rank::LEVELS.len() - 1);
    if !mandatory || steps == 0 {
        return Rank::LEVELS[target];
    }
    for protected in [Rank::Five, Rank::Ten, Rank::King, Rank::Ace] {
        let index = protected.level_index().unwrap() as usize;
        if index > start && index < target {
            return protected;
        }
    }
    Rank::LEVELS[target]
}

pub fn level_steps_between(from: Rank, to: Rank) -> u8 {
    to.level_index()
        .unwrap()
        .saturating_sub(from.level_index().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandatory_levels_stop_skipped_promotions() {
        assert_eq!(level_after(Rank::Four, 3, true), Rank::Five);
        assert_eq!(level_after(Rank::Five, 3, true), Rank::Eight);
        assert_eq!(level_after(Rank::Nine, 3, true), Rank::Ten);
        assert_eq!(level_after(Rank::Queen, 2, true), Rank::King);
        assert_eq!(level_after(Rank::Four, 3, false), Rank::Seven);
    }

    #[test]
    fn trump_strength_skips_level_and_connects_special_pairs() {
        let trump = Trump::new(Rank::Ten, Some(Suit::Heart)).unwrap();
        assert_eq!(trump.strength(Card::suited(0, Suit::Heart, Rank::Jack)), 8);
        assert_eq!(trump.strength(Card::suited(0, Suit::Heart, Rank::Nine)), 7);
        assert_eq!(trump.strength(Card::suited(0, Suit::Heart, Rank::Ace)), 11);
        assert_eq!(trump.strength(Card::suited(0, Suit::Spade, Rank::Ten)), 12);
        assert_eq!(trump.strength(Card::suited(0, Suit::Heart, Rank::Ten)), 13);
        assert_eq!(trump.strength(Card::small_joker(0)), 14);
        assert_eq!(trump.strength(Card::big_joker(0)), 15);
    }

    #[test]
    fn constant_two_sits_between_level_cards_and_trump_ace() {
        let trump = Trump::new(Rank::Ten, Some(Suit::Heart))
            .unwrap()
            .with_constant_trump(true);
        let ordered = [
            Card::suited(0, Suit::Heart, Rank::Ace),
            Card::suited(0, Suit::Spade, Rank::Two),
            Card::suited(0, Suit::Heart, Rank::Two),
            Card::suited(0, Suit::Spade, Rank::Ten),
            Card::suited(0, Suit::Heart, Rank::Ten),
            Card::small_joker(0),
            Card::big_joker(0),
        ];
        assert_eq!(
            ordered.map(|card| trump.strength(card)),
            [10, 11, 12, 13, 14, 15, 16]
        );
        assert!(ordered.into_iter().all(|card| trump.is_trump(card)));
    }

    #[test]
    fn deck_count_controls_hand_kitty_and_scoring_bands() {
        let three = RuleSet {
            deck_count: 3,
            ..RuleSet::default()
        };
        assert_eq!((three.hand_size(), three.kitty_size()), (39, 6));
        assert_eq!((three.score_step(), three.takeover_score()), (60, 120));

        let four = RuleSet {
            deck_count: 4,
            ..RuleSet::default()
        };
        assert!(four.validate().is_ok());
        assert_eq!((four.hand_size(), four.kitty_size()), (52, 8));
        assert_eq!((four.score_step(), four.takeover_score()), (80, 160));
        assert!(matches!(
            RuleSet {
                deck_count: 5,
                ..RuleSet::default()
            }
            .validate(),
            Err(RuleError::InvalidDeckCount(5))
        ));
    }
}
