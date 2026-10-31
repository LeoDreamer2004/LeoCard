//! 联机请求的通用拒绝原因与对应说明。

use crate::{
    BlackjackViolation, DoudizhuViolation, MahjongViolation, McGuessViolation, QiGui523Violation,
    RequestId, ShengjiViolation, TexasViolation, UnoViolation,
};
use leocard_blackjack::BlackjackGameError;
use leocard_doudizhu::DoudizhuGameError;
use leocard_game_common::PlayerId;
use leocard_mc_guess::McGuessGameError;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RejectReason {
    Request(RequestViolation),
    Player(PlayerViolation),
    Room(RoomViolation),
    Game(GameViolation),
}

impl fmt::Display for RejectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(reason) => reason.fmt(f),
            Self::Player(reason) => reason.fmt(f),
            Self::Room(reason) => reason.fmt(f),
            Self::Game(reason) => reason.fmt(f),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RequestViolation {
    ProtocolMismatch { expected: u16, received: u16 },
    RoomMismatch,
    DuplicateRequest { last_seen: RequestId },
    StaleRequest { last_seen: RequestId },
}

impl fmt::Display for RequestViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtocolMismatch { expected, received } => {
                write!(f, "协议版本不匹配：房主要求 {expected}，当前为 {received}")
            }
            Self::RoomMismatch => f.write_str("请求不属于当前房间"),
            Self::DuplicateRequest { .. } => f.write_str("请勿重复发送同一请求"),
            Self::StaleRequest { .. } => f.write_str("请求已过期，请重试"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlayerViolation {
    AlreadyJoined,
    NotJoined,
    NameEmpty,
    NameTooLong { max_chars: u16 },
    InvalidIdentityProof,
    InvalidAvatar,
    AvatarAlreadySet,
    InvalidAchievements,
}

impl fmt::Display for PlayerViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AlreadyJoined => "你已经加入房间",
            Self::NotJoined => "请先加入房间",
            Self::NameEmpty => "玩家名称不能为空",
            Self::NameTooLong { max_chars } => {
                return write!(f, "玩家名称不能超过 {max_chars} 个字符");
            }
            Self::InvalidIdentityProof => "玩家身份签名无效，请重新生成或恢复玩家档案",
            Self::InvalidAvatar => "头像数据无效",
            Self::AvatarAlreadySet => "本次连接已经上传过头像",
            Self::InvalidAchievements => "成就数量或条目无效",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum RoomViolation {
    RoomFull,
    InvalidSeat,
    SeatTaken,
    MustSelectSeat,
    OnlyHostCanConfigure,
    OnlyHostCanStart,
    OnlyHostCanReturnToLobby,
    OnlyHostCanCloseRoom,
    NotEnoughPlayers { minimum: u8, actual: u8 },
    WaitingForPlayers { expected: u8, actual: u8 },
    PlayersNotReady { players: Vec<PlayerId> },
    InvalidChatMessage,
    CannotInteractWithSelf,
    SpectatorCannotAct,
    OnlyHostCanKick,
    CannotKickPlayer,
}

impl fmt::Display for RoomViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RoomFull => "房间人数已满",
            Self::InvalidSeat => "座位编号无效",
            Self::SeatTaken => "这个座位已经有人了",
            Self::MustSelectSeat => "必须先选择座位",
            Self::OnlyHostCanConfigure => "只有房主可以修改游戏配置",
            Self::OnlyHostCanStart => "只有房主可以开始游戏",
            Self::OnlyHostCanReturnToLobby => "只有房主可以返回大厅",
            Self::OnlyHostCanCloseRoom => "只有房主可以关闭房间",
            Self::NotEnoughPlayers { minimum, .. } => {
                return write!(f, "至少需要 {minimum} 名玩家才能开始");
            }
            Self::WaitingForPlayers { .. } => "人数尚未到齐",
            Self::PlayersNotReady { .. } => "仍有玩家没有准备",
            Self::InvalidChatMessage => "聊天消息为空、过长或快捷语音无效",
            Self::CannotInteractWithSelf => "不能和自己互动",
            Self::SpectatorCannotAct => "当前正在观战，请等待下一场入座",
            Self::OnlyHostCanKick => "只有房主可以踢出玩家",
            Self::CannotKickPlayer => "只能踢出其他真人玩家",
        })
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GameViolation {
    InvalidRuleConfiguration,
    DeveloperFeatureUnavailable,
    WrongGame,
    GameNotStarted,
    GameNotFinished,
    GameAlreadyStarted,
    QiGui523(QiGui523Violation),
    Texas(TexasViolation),
    Shengji(ShengjiViolation),
    Uno(UnoViolation),
    Mahjong(MahjongViolation),
    McGuess(McGuessViolation),
    Blackjack(BlackjackViolation),
    Doudizhu(DoudizhuViolation),
}

impl fmt::Display for GameViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRuleConfiguration => f.write_str("游戏规则配置无效"),
            Self::DeveloperFeatureUnavailable => f.write_str("房主程序没有启用开发者功能"),
            Self::WrongGame => f.write_str("该命令不属于当前房间游戏"),
            Self::GameNotStarted => f.write_str("游戏尚未开始"),
            Self::GameNotFinished => f.write_str("游戏尚未结束"),
            Self::GameAlreadyStarted => f.write_str("游戏已经开始"),
            Self::QiGui523(reason) => reason.fmt(f),
            Self::Texas(reason) => reason.fmt(f),
            Self::Shengji(reason) => reason.fmt(f),
            Self::Uno(reason) => reason.fmt(f),
            Self::Mahjong(reason) => reason.fmt(f),
            Self::McGuess(reason) => reason.fmt(f),
            Self::Blackjack(reason) => reason.fmt(f),
            Self::Doudizhu(reason) => reason.fmt(f),
        }
    }
}

impl From<QiGui523Violation> for GameViolation {
    fn from(value: QiGui523Violation) -> Self {
        Self::QiGui523(value)
    }
}

impl From<TexasViolation> for GameViolation {
    fn from(value: TexasViolation) -> Self {
        Self::Texas(value)
    }
}

impl From<ShengjiViolation> for GameViolation {
    fn from(value: ShengjiViolation) -> Self {
        Self::Shengji(value)
    }
}

impl From<UnoViolation> for GameViolation {
    fn from(value: UnoViolation) -> Self {
        Self::Uno(value)
    }
}

impl From<MahjongViolation> for GameViolation {
    fn from(value: MahjongViolation) -> Self {
        Self::Mahjong(value)
    }
}

impl From<McGuessViolation> for GameViolation {
    fn from(value: McGuessViolation) -> Self {
        Self::McGuess(value)
    }
}

impl From<BlackjackViolation> for GameViolation {
    fn from(value: BlackjackViolation) -> Self {
        Self::Blackjack(value)
    }
}

impl From<DoudizhuViolation> for GameViolation {
    fn from(value: DoudizhuViolation) -> Self {
        Self::Doudizhu(value)
    }
}

impl From<BlackjackGameError> for GameViolation {
    fn from(value: BlackjackGameError) -> Self {
        Self::Blackjack(value.into())
    }
}

impl From<DoudizhuGameError> for GameViolation {
    fn from(value: DoudizhuGameError) -> Self {
        Self::Doudizhu(value.into())
    }
}

impl From<McGuessGameError> for GameViolation {
    fn from(value: McGuessGameError) -> Self {
        Self::McGuess(value.into())
    }
}
