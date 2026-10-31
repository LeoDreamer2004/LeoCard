use crate::{BlackjackCard, BlackjackHandValue, BlackjackRuleSet};
use leocard_game_common::PlayerId;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlackjackAction {
    Bet { amount: u32 },
    Hit,
    Stand,
    DoubleDown,
    Split,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlackjackPhase {
    Betting,
    Playing,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlackjackHandStatus {
    Playing,
    Standing,
    Bust,
    Blackjack,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub enum BlackjackHandResult {
    Win,
    Loss,
    Push,
    Blackjack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackHandState {
    pub cards: Vec<BlackjackCard>,
    pub wager: u32,
    pub status: BlackjackHandStatus,
    pub split: bool,
    pub doubled: bool,
    pub result: Option<BlackjackHandResult>,
    /// 本手净收益，已经扣除原下注。
    pub net_chips: i32,
}

impl BlackjackHandState {
    pub fn value(&self) -> BlackjackHandValue {
        BlackjackHandValue::of(&self.cards)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackPlayerStatistics {
    pub rounds: u32,
    pub hands: u32,
    pub wins: u32,
    pub blackjacks: u32,
    pub busts: u32,
    pub doubles: u32,
    pub splits: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackPlayerState {
    pub stack: u32,
    pub hands: Vec<BlackjackHandState>,
    pub statistics: BlackjackPlayerStatistics,
    pub round_start_stack: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct BlackjackAvailableActions {
    pub minimum_bet: u32,
    pub maximum_bet: u32,
    pub hit: bool,
    pub stand: bool,
    pub double_down: bool,
    pub split: bool,
}

#[derive(Clone, Debug)]
pub struct BlackjackGameState {
    pub(super) rules: BlackjackRuleSet,
    pub(super) players: Vec<BlackjackPlayerState>,
    pub(super) dealer: Vec<BlackjackCard>,
    pub(super) shoe: Vec<BlackjackCard>,
    pub(super) phase: BlackjackPhase,
    pub(super) round: u8,
    pub(super) current: Option<PlayerId>,
    pub(super) hand: usize,
}

impl BlackjackGameState {
    pub const fn rules(&self) -> BlackjackRuleSet {
        self.rules
    }

    pub fn players(&self) -> &[BlackjackPlayerState] {
        &self.players
    }

    pub const fn phase(&self) -> BlackjackPhase {
        self.phase
    }

    pub const fn round(&self) -> u8 {
        self.round
    }

    pub const fn current_player(&self) -> Option<PlayerId> {
        self.current
    }

    pub fn current_hand(&self) -> Option<usize> {
        (self.phase == BlackjackPhase::Playing).then_some(self.hand)
    }

    /// 行动阶段仅公开庄家明牌；暗牌及牌堆不通过此接口泄露。
    pub fn visible_dealer(&self) -> &[BlackjackCard] {
        if self.phase == BlackjackPhase::Complete {
            &self.dealer
        } else {
            &self.dealer[..self.dealer.len().min(1)]
        }
    }

    pub fn dealer_has_hidden_card(&self) -> bool {
        self.phase == BlackjackPhase::Playing && self.dealer.len() > 1
    }

    pub fn match_complete(&self) -> bool {
        self.phase == BlackjackPhase::Complete
            && (self.round >= self.rules.rounds
                || self
                    .players
                    .iter()
                    .all(|player| player.stack < self.rules.minimum_bet))
    }
}
