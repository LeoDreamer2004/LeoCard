use crate::{MatchId, PlayerPublicInfo, PlayerReferenceChange, SeatId};
use leocard_game_common::PlayerId;
use leocard_guandan::{
    GuandanAction, GuandanCard, GuandanGameError, GuandanGameResult, GuandanPhase, GuandanPlay,
    GuandanRuleSet, GuandanTributeAction,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuandanCommand {
    UpdateRules { rules: GuandanRuleSet },
    SetAutoPlay { enabled: bool },
    Act(GuandanAction),
}

/// Public facts only. Tribute card identities are carried by recipient-specific snapshots.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuandanEvent {
    ActionApplied {
        player: PlayerId,
        action: GuandanPublicAction,
    },
    TributeSubmitted {
        player: PlayerId,
    },
    HandFinished {
        result: GuandanGameResult,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuandanPublicAction {
    Play(Vec<GuandanCard>),
    Pass,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuandanSnapshot {
    pub match_id: MatchId,
    pub host_port: u16,
    pub you: PlayerId,
    pub host: PlayerId,
    pub players: Vec<GuandanPlayerView>,
    pub rules: GuandanRuleSet,
    pub level: u8,
    pub team_levels: [u8; 2],
    pub hand_number: u32,
    pub match_complete: bool,
    pub reference_changes: Vec<PlayerReferenceChange>,
    pub phase: GuandanPhase,
    pub current: Option<PlayerId>,
    pub tribute_action: Option<GuandanTributeAction>,
    pub received_tribute: Option<GuandanCard>,
    pub tribute_pairs: Vec<(PlayerId, PlayerId)>,
    pub tribute_resisted: bool,
    pub your_hand: Vec<GuandanCard>,
    /// All publicly played cards in this hand, including the current trick.
    pub played_cards: Vec<GuandanCard>,
    pub table: Option<(PlayerId, GuandanPlay)>,
    pub last_actions: [Option<GuandanPublicAction>; 4],
    pub result: Option<GuandanGameResult>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuandanPlayerView {
    pub player: PlayerPublicInfo,
    pub seat: SeatId,
    pub hand_len: u16,
    pub ready: bool,
    pub connected: bool,
    pub auto_play: bool,
}

/// Protocol-facing rejection, independent of the internal rule error layout.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuandanViolation {
    InvalidRules,
    InvalidDeck,
    InvalidPlayer,
    NotYourTurn,
    GameAlreadyFinished,
    MustLead,
    CardNotInHand,
    InvalidPlay,
    CannotBeat,
    WrongPhase,
    NotHighestTribute,
    InvalidReturnTribute,
}

impl From<GuandanGameError> for GuandanViolation {
    fn from(error: GuandanGameError) -> Self {
        match error {
            GuandanGameError::InvalidRules => Self::InvalidRules,
            GuandanGameError::InvalidDeckSize { .. } | GuandanGameError::InvalidDeckContents => {
                Self::InvalidDeck
            }
            GuandanGameError::InvalidPlayer(_) => Self::InvalidPlayer,
            GuandanGameError::NotPlayersTurn { .. } => Self::NotYourTurn,
            GuandanGameError::GameAlreadyFinished => Self::GameAlreadyFinished,
            GuandanGameError::MustLeadWithCards => Self::MustLead,
            GuandanGameError::CardNotInHand(_) => Self::CardNotInHand,
            GuandanGameError::InvalidPlay(_) => Self::InvalidPlay,
            GuandanGameError::PlayDoesNotBeatCurrent => Self::CannotBeat,
            GuandanGameError::WrongPhase => Self::WrongPhase,
            GuandanGameError::NotHighestTribute => Self::NotHighestTribute,
            GuandanGameError::InvalidReturnTribute => Self::InvalidReturnTribute,
        }
    }
}

impl fmt::Display for GuandanViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules => f.write_str("掼蛋规则配置无效"),
            Self::InvalidDeck => f.write_str("掼蛋牌堆无效"),
            Self::InvalidPlayer => f.write_str("玩家无效"),
            Self::NotYourTurn => f.write_str("还没轮到你"),
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::MustLead => f.write_str("领出时必须出牌"),
            Self::CardNotInHand => f.write_str("手牌中没有所选的牌"),
            Self::InvalidPlay => f.write_str("牌型无效"),
            Self::CannotBeat => f.write_str("所出牌不能压过桌面牌"),
            Self::WrongPhase => f.write_str("当前阶段不允许此操作"),
            Self::NotHighestTribute => f.write_str("必须进贡最大的牌"),
            Self::InvalidReturnTribute => f.write_str("还贡牌不符合规则"),
        }
    }
}
