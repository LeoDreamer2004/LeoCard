use crate::{ClassifiedPlay, PlayError, QiGuiCard, QiGuiRuleSet, RuleError};
use std::collections::VecDeque;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct QiGuiPlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub(super) id: QiGuiPlayerId,
    pub(super) hand: Vec<QiGuiCard>,
    pub(super) score: u32,
}

impl PlayerState {
    pub fn id(&self) -> QiGuiPlayerId {
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
pub struct StartingCard {
    pub player: QiGuiPlayerId,
    pub card: QiGuiCard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayRecord {
    Played {
        player: QiGuiPlayerId,
        play: ClassifiedPlay,
    },
    Passed {
        player: QiGuiPlayerId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrickState {
    pub(super) leader: QiGuiPlayerId,
    pub(super) current_player: QiGuiPlayerId,
    pub(super) winning_player: Option<QiGuiPlayerId>,
    pub(super) winning_play: Option<ClassifiedPlay>,
    pub(super) records: Vec<PlayRecord>,
    pub(super) table_points: u32,
    pub(super) passes_after_winning_play: usize,
}

impl TrickState {
    pub(super) fn new(leader: QiGuiPlayerId) -> Self {
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

    pub fn leader(&self) -> QiGuiPlayerId {
        self.leader
    }

    pub fn current_player(&self) -> QiGuiPlayerId {
        self.current_player
    }

    pub fn winning_player(&self) -> Option<QiGuiPlayerId> {
        self.winning_player
    }

    pub fn winning_play(&self) -> Option<&ClassifiedPlay> {
        self.winning_play.as_ref()
    }

    pub fn records(&self) -> &[PlayRecord] {
        &self.records
    }

    pub fn table_points(&self) -> u32 {
        self.table_points
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameResult {
    /// 摸牌堆耗尽后，第一个出完手牌的玩家。
    pub finisher: QiGuiPlayerId,
    pub scores: Vec<u32>,
    pub captured_hand_points: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Playing,
    Finished(GameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    Played {
        player: QiGuiPlayerId,
        next_player: QiGuiPlayerId,
    },
    Passed {
        player: QiGuiPlayerId,
        next_player: QiGuiPlayerId,
    },
    TrickCompleted {
        winner: QiGuiPlayerId,
        points: u32,
        next_player: QiGuiPlayerId,
        cards_drawn: usize,
    },
    GameFinished(GameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(QiGuiPlayerId),
    NotPlayersTurn {
        expected: QiGuiPlayerId,
        actual: QiGuiPlayerId,
    },
    GameAlreadyFinished,
    MustLeadWithCards,
    CardNotInHand(QiGuiCard),
    InvalidPlay(PlayError),
    PlayDoesNotBeatCurrent,
}

impl fmt::Display for GameError {
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

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

impl From<PlayError> for GameError {
    fn from(value: PlayError) -> Self {
        Self::InvalidPlay(value)
    }
}

/// 权威游戏状态。所有修改都只能通过 [`GameState::play_cards`] 或
/// [`GameState::pass`] 完成。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub(super) rules: QiGuiRuleSet,
    pub(super) players: Vec<PlayerState>,
    pub(super) draw_pile: VecDeque<QiGuiCard>,
    pub(super) starting_card: StartingCard,
    pub(super) trick: Option<TrickState>,
    pub(super) phase: Phase,
}

impl GameState {
    pub fn rules(&self) -> &QiGuiRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, id: QiGuiPlayerId) -> Option<&PlayerState> {
        self.players.get(id.0)
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn starting_card(&self) -> StartingCard {
        self.starting_card
    }

    pub fn trick(&self) -> Option<&TrickState> {
        self.trick.as_ref()
    }

    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// 开发构建专用：完整替换某名玩家的手牌，不检查牌副范围或实体牌重复。
    #[cfg(feature = "developer")]
    pub fn replace_player_hand(
        &mut self,
        player: QiGuiPlayerId,
        cards: Vec<QiGuiCard>,
    ) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            return Err(GameError::GameAlreadyFinished);
        }
        if player.0 >= self.players.len() {
            return Err(GameError::InvalidPlayer(player));
        }
        if cards.is_empty() {
            return Err(GameError::InvalidDeckContents);
        }
        self.players[player.0].hand = cards;
        self.players[player.0].hand.sort_by(QiGuiCard::display_cmp);
        Ok(())
    }
}
