use crate::{Card, Play, RuleSet, build_deck};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    Bid(u8),
    Play(Vec<Card>),
    Pass,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Phase {
    Bidding,
    Playing,
    Redeal,
    Finished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResultView {
    pub winner: u8,
    pub landlord_won: bool,
    pub spring: bool,
    pub scores: Vec<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameError {
    InvalidRules,
    InvalidDeck,
    InvalidPlayer,
    NotYourTurn,
    WrongPhase,
    InvalidBid,
    InvalidPattern,
    CardNotInHand,
    MustPlay,
    CannotBeat,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRules => "游戏轮数必须为 1 至 5 轮",
            Self::InvalidDeck => "牌堆无效",
            Self::InvalidPlayer => "玩家身份无效",
            Self::NotYourTurn => "还没有轮到你",
            Self::WrongPhase => "当前阶段不能执行此操作",
            Self::InvalidBid => "叫分必须高于当前分数，且不超过三分",
            Self::InvalidPattern => "所选牌不能组成合法牌型",
            Self::CardNotInHand => "选择的牌不在手中",
            Self::MustPlay => "领出时必须出牌",
            Self::CannotBeat => "这手牌无法压过当前牌",
        })
    }
}

impl Error for GameError {}

#[derive(Clone, Debug)]
pub struct GameState {
    pub rules: RuleSet,
    pub phase: Phase,
    pub current: u8,
    pub landlord: Option<u8>,
    pub highest_bid: u8,
    pub bids: Vec<Option<u8>>,
    pub table: Option<(u8, Play)>,
    pub played: Vec<Card>,
    pub last_actions: Vec<Option<Action>>,
    pub multiplier: u32,
    pub result: Option<ResultView>,
    pub(super) hands: Vec<Vec<Card>>,
    pub(super) kitty: Vec<Card>,
    pub(super) passes: u8,
    pub(super) play_counts: Vec<u32>,
}

impl GameState {
    pub fn new(rules: RuleSet, deck: Vec<Card>, first: u8) -> Result<Self, GameError> {
        if !rules.is_valid() {
            return Err(GameError::InvalidRules);
        }
        let mut sorted = deck.clone();
        sorted.sort();
        if sorted != build_deck(rules.mode) {
            return Err(GameError::InvalidDeck);
        }
        let n = usize::from(rules.mode.players());
        if usize::from(first) >= n {
            return Err(GameError::InvalidPlayer);
        }
        let hand_end = deck.len() - rules.mode.kitty_len();
        let mut hands = vec![Vec::new(); n];
        for (i, card) in deck[..hand_end].iter().enumerate() {
            hands[i % n].push(*card);
        }
        for hand in &mut hands {
            hand.sort_by_key(|c| (c.rank(), c.0));
        }
        Ok(Self {
            rules,
            phase: Phase::Bidding,
            current: first,
            landlord: None,
            highest_bid: 0,
            bids: vec![None; n],
            table: None,
            played: Vec::new(),
            last_actions: vec![None; n],
            multiplier: 1,
            result: None,
            hands,
            kitty: deck[hand_end..].to_vec(),
            passes: 0,
            play_counts: vec![0; n],
        })
    }

    pub fn hand(&self, player: u8) -> Option<&[Card]> {
        self.hands.get(usize::from(player)).map(Vec::as_slice)
    }

    pub fn hand_lengths(&self) -> Vec<u16> {
        self.hands.iter().map(|h| h.len() as u16).collect()
    }

    pub fn public_kitty(&self) -> &[Card] {
        if matches!(self.phase, Phase::Playing | Phase::Finished) {
            &self.kitty
        } else {
            &[]
        }
    }

    pub fn revealed_hands(&self) -> Vec<Vec<Card>> {
        if self.phase == Phase::Finished {
            self.hands.clone()
        } else {
            Vec::new()
        }
    }
}
