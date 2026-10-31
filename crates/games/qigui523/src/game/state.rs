use crate::{QiGuiCard, QiGuiClassifiedPlay, QiGuiPlayError, QiGuiRuleError, QiGuiRuleSet};
use leocard_game_common::{DeckError, PlayerId};

use std::{collections::VecDeque, error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGuiPlayerState {
    pub(super) id: PlayerId,
    pub(super) hand: Vec<QiGuiCard>,
    pub(super) score: u32,
}

impl QiGuiPlayerState {
    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[QiGuiCard] {
        &self.hand
    }

    pub fn score(&self) -> u32 {
        self.score
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QiGuiStartingCard {
    pub player: PlayerId,
    pub card: QiGuiCard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QiGuiPlayRecord {
    Played {
        player: PlayerId,
        play: QiGuiClassifiedPlay,
    },
    Passed {
        player: PlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGuiTrickState {
    pub(super) leader: PlayerId,
    pub(super) current_player: PlayerId,
    pub(super) winning_player: Option<PlayerId>,
    pub(super) winning_play: Option<QiGuiClassifiedPlay>,
    pub(super) records: Vec<QiGuiPlayRecord>,
    pub(super) table_points: u32,
    pub(super) passes_after_winning_play: usize,
}

impl QiGuiTrickState {
    pub(super) fn new(leader: PlayerId) -> Self {
        Self {
            leader,
            current_player: leader,
            winning_player: None,
            winning_play: None,
            records: Vec::new(),
            table_points: 0,
            passes_after_winning_play: 0,
        }
    }

    pub fn leader(&self) -> PlayerId {
        self.leader
    }

    pub fn current_player(&self) -> PlayerId {
        self.current_player
    }

    pub fn winning_player(&self) -> Option<PlayerId> {
        self.winning_player
    }

    pub fn winning_play(&self) -> Option<&QiGuiClassifiedPlay> {
        self.winning_play.as_ref()
    }

    pub fn records(&self) -> &[QiGuiPlayRecord] {
        &self.records
    }

    pub fn table_points(&self) -> u32 {
        self.table_points
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGuiGameResult {
    /// 摸牌堆耗尽后，第一个出完手牌的玩家。
    pub finisher: PlayerId,
    pub scores: Vec<u32>,
    pub captured_hand_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QiGuiPhase {
    Playing,
    Finished(QiGuiGameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QiGuiActionOutcome {
    Played {
        player: PlayerId,
        next_player: PlayerId,
    },
    Passed {
        player: PlayerId,
        next_player: PlayerId,
    },
    TrickCompleted {
        winner: PlayerId,
        points: u32,
        next_player: PlayerId,
        cards_drawn: usize,
    },
    GameFinished(QiGuiGameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QiGuiGameError {
    InvalidRules(QiGuiRuleError),
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
    CardNotInHand(QiGuiCard),
    InvalidPlay(QiGuiPlayError),
    PlayDoesNotBeatCurrent,
}

impl fmt::Display for QiGuiGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或不属于当前副数的牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}；当前应由 {:?} 操作", actual, expected)
            }
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::MustLeadWithCards => f.write_str("本轮首家不能跳过，必须出牌"),
            Self::CardNotInHand(card) => write!(f, "玩家手中没有这张牌：{card}"),
            Self::InvalidPlay(error) => error.fmt(f),
            Self::PlayDoesNotBeatCurrent => f.write_str("所出牌型不能压过当前最大牌型"),
        }
    }
}

impl Error for QiGuiGameError {}

impl From<DeckError> for QiGuiGameError {
    fn from(value: DeckError) -> Self {
        match value {
            DeckError::InvalidSize { expected, actual } => {
                Self::InvalidDeckSize { expected, actual }
            }
            DeckError::InvalidContents => Self::InvalidDeckContents,
        }
    }
}

impl From<QiGuiRuleError> for QiGuiGameError {
    fn from(value: QiGuiRuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<QiGuiPlayError> for QiGuiGameError {
    fn from(value: QiGuiPlayError) -> Self {
        Self::InvalidPlay(value)
    }
}

/// 权威游戏状态。所有修改都只能通过 [`QiGuiGameState::play_cards`] 或
/// [`QiGuiGameState::pass`] 完成。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QiGuiGameState {
    pub(super) rules: QiGuiRuleSet,
    pub(super) players: Vec<QiGuiPlayerState>,
    pub(super) draw_pile: VecDeque<QiGuiCard>,
    pub(super) played_cards: Vec<QiGuiCard>,
    pub(super) starting_card: QiGuiStartingCard,
    pub(super) trick: Option<QiGuiTrickState>,
    pub(super) phase: QiGuiPhase,
}

impl QiGuiGameState {
    pub fn rules(&self) -> &QiGuiRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[QiGuiPlayerState] {
        &self.players
    }

    pub fn player(&self, id: PlayerId) -> Option<&QiGuiPlayerState> {
        self.players.get(id.index())
    }

    /// 本局实际公开打出的实体牌，包含已经收走的轮次。
    pub fn played_cards(&self) -> &[QiGuiCard] {
        &self.played_cards
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    /// Total scoring value of the deck, including captured and unplayed cards.
    pub fn total_points(&self) -> u32 {
        self.trick.as_ref().map_or(0, |trick| trick.table_points)
            + self.players.iter().map(|player| player.score).sum::<u32>()
            + self
                .players
                .iter()
                .flat_map(|player| &player.hand)
                .chain(&self.draw_pile)
                .map(|card| u32::from(card.score()))
                .sum::<u32>()
    }

    pub fn starting_card(&self) -> QiGuiStartingCard {
        self.starting_card
    }

    pub fn trick(&self) -> Option<&QiGuiTrickState> {
        self.trick.as_ref()
    }

    pub fn phase(&self) -> &QiGuiPhase {
        &self.phase
    }

    /// 开发构建专用：完整替换某名玩家的手牌，不检查牌副范围或实体牌重复。
    #[cfg(feature = "developer")]
    pub fn replace_player_hand(
        &mut self,
        player: PlayerId,
        cards: Vec<QiGuiCard>,
    ) -> Result<(), QiGuiGameError> {
        if matches!(self.phase, QiGuiPhase::Finished(_)) {
            return Err(QiGuiGameError::GameAlreadyFinished);
        }
        if player.index() >= self.players.len() {
            return Err(QiGuiGameError::InvalidPlayer(player));
        }
        if cards.is_empty() {
            return Err(QiGuiGameError::InvalidDeckContents);
        }
        self.players[player.index()].hand = cards;
        self.players[player.index()]
            .hand
            .sort_by(QiGuiCard::display_cmp);
        Ok(())
    }
}
