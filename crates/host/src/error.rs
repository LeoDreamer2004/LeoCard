//! 按具体游戏保留规则、牌堆和运行错误的来源。

use leocard_blackjack::GameError as BlackjackError;
use leocard_mahjong::{GameError as MahjongError, RuleError as MahjongRuleError};
use leocard_minecraft_guess::GameError as MinecraftGuessError;
use leocard_qigui523::{GameError as QiGui523Error, RuleError as QiGui523RuleError};
use leocard_shengji::{GameError as ShengjiError, RuleError as ShengjiRuleError};
use leocard_texas_holdem::{GameError as TexasHoldemError, RuleError as TexasHoldemRuleError};
use leocard_uno::{GameError as UnoError, RuleError as UnoRuleError};
use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    Blackjack(BlackjackError),
    Mahjong(MahjongError),
    MinecraftGuess(MinecraftGuessError),
    QiGui523(QiGui523Error),
    Shengji(ShengjiError),
    TexasHoldem(TexasHoldemError),
    Uno(UnoError),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blackjack(error) => error.fmt(f),
            Self::Mahjong(error) => error.fmt(f),
            Self::MinecraftGuess(error) => error.fmt(f),
            Self::QiGui523(error) => error.fmt(f),
            Self::Shengji(error) => error.fmt(f),
            Self::TexasHoldem(error) => error.fmt(f),
            Self::Uno(error) => error.fmt(f),
        }
    }
}

impl Error for HostError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Blackjack(error) => Some(error),
            Self::Mahjong(error) => Some(error),
            Self::MinecraftGuess(error) => Some(error),
            Self::QiGui523(error) => Some(error),
            Self::Shengji(error) => Some(error),
            Self::TexasHoldem(error) => Some(error),
            Self::Uno(error) => Some(error),
        }
    }
}

impl From<BlackjackError> for HostError {
    fn from(error: BlackjackError) -> Self {
        Self::Blackjack(error)
    }
}

impl From<MahjongError> for HostError {
    fn from(error: MahjongError) -> Self {
        Self::Mahjong(error)
    }
}

impl From<MahjongRuleError> for HostError {
    fn from(error: MahjongRuleError) -> Self {
        Self::Mahjong(error.into())
    }
}

impl From<MinecraftGuessError> for HostError {
    fn from(error: MinecraftGuessError) -> Self {
        Self::MinecraftGuess(error)
    }
}

impl From<QiGui523Error> for HostError {
    fn from(error: QiGui523Error) -> Self {
        Self::QiGui523(error)
    }
}

impl From<QiGui523RuleError> for HostError {
    fn from(error: QiGui523RuleError) -> Self {
        Self::QiGui523(error.into())
    }
}

impl From<ShengjiError> for HostError {
    fn from(error: ShengjiError) -> Self {
        Self::Shengji(error)
    }
}

impl From<ShengjiRuleError> for HostError {
    fn from(error: ShengjiRuleError) -> Self {
        Self::Shengji(error.into())
    }
}

impl From<TexasHoldemError> for HostError {
    fn from(error: TexasHoldemError) -> Self {
        Self::TexasHoldem(error)
    }
}

impl From<TexasHoldemRuleError> for HostError {
    fn from(error: TexasHoldemRuleError) -> Self {
        Self::TexasHoldem(error.into())
    }
}

impl From<UnoError> for HostError {
    fn from(error: UnoError) -> Self {
        Self::Uno(error)
    }
}

impl From<UnoRuleError> for HostError {
    fn from(error: UnoRuleError) -> Self {
        Self::Uno(error.into())
    }
}
