use crate::{GuandanCard, GuandanPlay, GuandanPlayError, GuandanRuleSet};
use leocard_game_common::PlayerId;
use std::{error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuandanPlayerState {
    pub(super) id: PlayerId,
    pub(super) hand: Vec<GuandanCard>,
}

impl GuandanPlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[GuandanCard] {
        &self.hand
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuandanGameResult {
    /// Only this many leading placements are known. A double finish ends as
    /// soon as the partners take first and second; the other two are unranked.
    pub known_positions: u8,
    pub finishing_order: [PlayerId; 4],
    pub winning_team: u8,
    pub level_gain: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuandanPhase {
    Tribute,
    Playing,
    Finished(GuandanGameResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuandanTributeAction {
    Tribute,
    ReturnTribute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuandanAction {
    Tribute(GuandanCard),
    ReturnTribute(GuandanCard),
    Play(Vec<GuandanCard>),
    Pass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuandanActionOutcome {
    TributeSubmitted {
        player: PlayerId,
        next_player: PlayerId,
    },
    TributeReturned {
        player: PlayerId,
        next_player: PlayerId,
    },
    Played {
        player: PlayerId,
        play: GuandanPlay,
        next_player: PlayerId,
    },
    Passed {
        player: PlayerId,
        next_player: PlayerId,
    },
    TrickCompleted {
        winner: PlayerId,
        next_player: PlayerId,
    },
    GameFinished(GuandanGameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuandanGameError {
    InvalidRules,
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(PlayerId),
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand(GuandanCard),
    InvalidPlay(GuandanPlayError),
    PlayDoesNotBeatCurrent,
    WrongPhase,
    NotHighestTribute,
    InvalidReturnTribute,
}

impl fmt::Display for GuandanGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules => f.write_str("掼蛋规则配置无效"),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆应为 {expected} 张，实际为 {actual} 张")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}，当前应由 {:?} 操作", actual, expected)
            }
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::MustLeadWithCards => f.write_str("领出时不能过牌"),
            Self::CardNotInHand(card) => write!(f, "手中没有这张牌：{card:?}"),
            Self::InvalidPlay(error) => error.fmt(f),
            Self::PlayDoesNotBeatCurrent => f.write_str("所出牌不能压过当前牌型"),
            Self::WrongPhase => f.write_str("当前阶段不允许此操作"),
            Self::NotHighestTribute => f.write_str("进贡时必须交出手中最大的牌"),
            Self::InvalidReturnTribute => f.write_str("还贡牌必须不大于十"),
        }
    }
}

impl Error for GuandanGameError {}

impl From<GuandanPlayError> for GuandanGameError {
    fn from(value: GuandanPlayError) -> Self {
        Self::InvalidPlay(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TributeExchange {
    pub giver: PlayerId,
    pub receiver: PlayerId,
    pub offered: Option<GuandanCard>,
    pub returned: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuandanGameState {
    pub(super) rules: GuandanRuleSet,
    pub(super) players: [GuandanPlayerState; 4],
    pub(super) phase: GuandanPhase,
    pub(super) starter: PlayerId,
    pub(super) current_player: PlayerId,
    pub(super) winning_player: Option<PlayerId>,
    pub(super) winning_play: Option<GuandanPlay>,
    pub(super) passes: usize,
    pub(super) finishing_order: Vec<PlayerId>,
    pub(super) played_cards: Vec<GuandanCard>,
    pub(super) tribute_pairs: Vec<(PlayerId, PlayerId)>,
    pub(super) exchanges: Vec<TributeExchange>,
    pub(super) tribute_resisted: bool,
    pub(super) last_actions: [Option<GuandanAction>; 4],
}

impl GuandanGameState {
    pub const fn rules(&self) -> &GuandanRuleSet {
        &self.rules
    }

    pub const fn players(&self) -> &[GuandanPlayerState; 4] {
        &self.players
    }

    pub fn player(&self, player: PlayerId) -> Option<&GuandanPlayerState> {
        self.players.get(player.index())
    }

    pub const fn phase(&self) -> &GuandanPhase {
        &self.phase
    }

    pub fn current_player(&self) -> Option<PlayerId> {
        if matches!(self.phase, GuandanPhase::Finished(_)) {
            None
        } else {
            Some(self.current_player)
        }
    }

    pub const fn winning_player(&self) -> Option<PlayerId> {
        self.winning_player
    }

    pub const fn winning_play(&self) -> Option<&GuandanPlay> {
        self.winning_play.as_ref()
    }

    pub fn finishing_order(&self) -> &[PlayerId] {
        &self.finishing_order
    }

    pub fn played_cards(&self) -> &[GuandanCard] {
        &self.played_cards
    }

    pub fn tribute_pairs(&self) -> &[(PlayerId, PlayerId)] {
        &self.tribute_pairs
    }

    pub const fn tribute_resisted(&self) -> bool {
        self.tribute_resisted
    }

    pub fn pending_tribute_action(&self) -> Option<GuandanTributeAction> {
        if !matches!(self.phase, GuandanPhase::Tribute) {
            return None;
        }
        self.exchanges
            .iter()
            .find(|exchange| !exchange.returned)
            .map(|exchange| {
                if exchange.offered.is_some() {
                    GuandanTributeAction::ReturnTribute
                } else {
                    GuandanTributeAction::Tribute
                }
            })
    }

    pub(crate) fn pending_offered_card(&self) -> Option<GuandanCard> {
        self.exchanges
            .iter()
            .find(|exchange| !exchange.returned)
            .and_then(|exchange| exchange.offered)
    }

    /// The offered card is visible only to its receiver while a return is pending.
    pub fn received_tribute_for(&self, player: PlayerId) -> Option<GuandanCard> {
        self.exchanges
            .iter()
            .find(|exchange| exchange.receiver == player && !exchange.returned)
            .and_then(|exchange| exchange.offered)
    }

    pub const fn last_actions(&self) -> &[Option<GuandanAction>; 4] {
        &self.last_actions
    }
}
