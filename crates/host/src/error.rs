//! 按具体游戏保留规则、牌堆和运行错误的来源。

use leocard_blackjack::BlackjackGameError;
use leocard_doudizhu::DoudizhuGameError;
use leocard_mahjong::{MahjongGameError, MahjongRuleError};
use leocard_mc_guess::McGuessGameError;
use leocard_qigui523::{QiGuiGameError, QiGuiRuleError};
use leocard_shengji::{ShengjiGameError, ShengjiRuleError};
use leocard_texas_holdem::{TexasGameError, TexasRuleError};
use leocard_uno::{UnoGameError, UnoRuleError};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    Doudizhu(DoudizhuGameError),
    Blackjack(BlackjackGameError),
    Mahjong(MahjongGameError),
    McGuess(McGuessGameError),
    QiGui523(QiGuiGameError),
    Shengji(ShengjiGameError),
    Texas(TexasGameError),
    Uno(UnoGameError),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Doudizhu(error) => error.fmt(f),
            Self::Blackjack(error) => error.fmt(f),
            Self::Mahjong(error) => error.fmt(f),
            Self::McGuess(error) => error.fmt(f),
            Self::QiGui523(error) => error.fmt(f),
            Self::Shengji(error) => error.fmt(f),
            Self::Texas(error) => error.fmt(f),
            Self::Uno(error) => error.fmt(f),
        }
    }
}

impl Error for HostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Doudizhu(error) => Some(error),
            Self::Blackjack(error) => Some(error),
            Self::Mahjong(error) => Some(error),
            Self::McGuess(error) => Some(error),
            Self::QiGui523(error) => Some(error),
            Self::Shengji(error) => Some(error),
            Self::Texas(error) => Some(error),
            Self::Uno(error) => Some(error),
        }
    }
}

impl From<BlackjackGameError> for HostError {
    fn from(error: BlackjackGameError) -> Self {
        Self::Blackjack(error)
    }
}

impl From<MahjongGameError> for HostError {
    fn from(error: MahjongGameError) -> Self {
        Self::Mahjong(error)
    }
}

impl From<MahjongRuleError> for HostError {
    fn from(error: MahjongRuleError) -> Self {
        Self::Mahjong(error.into())
    }
}

impl From<McGuessGameError> for HostError {
    fn from(error: McGuessGameError) -> Self {
        Self::McGuess(error)
    }
}

impl From<QiGuiGameError> for HostError {
    fn from(error: QiGuiGameError) -> Self {
        Self::QiGui523(error)
    }
}

impl From<QiGuiRuleError> for HostError {
    fn from(error: QiGuiRuleError) -> Self {
        Self::QiGui523(error.into())
    }
}

impl From<ShengjiGameError> for HostError {
    fn from(error: ShengjiGameError) -> Self {
        Self::Shengji(error)
    }
}

impl From<ShengjiRuleError> for HostError {
    fn from(error: ShengjiRuleError) -> Self {
        Self::Shengji(error.into())
    }
}

impl From<TexasGameError> for HostError {
    fn from(error: TexasGameError) -> Self {
        Self::Texas(error)
    }
}

impl From<TexasRuleError> for HostError {
    fn from(error: TexasRuleError) -> Self {
        Self::Texas(error.into())
    }
}

impl From<UnoGameError> for HostError {
    fn from(error: UnoGameError) -> Self {
        Self::Uno(error)
    }
}

impl From<UnoRuleError> for HostError {
    fn from(error: UnoRuleError) -> Self {
        Self::Uno(error.into())
    }
}

impl From<DoudizhuGameError> for HostError {
    fn from(error: DoudizhuGameError) -> Self {
        Self::Doudizhu(error)
    }
}
