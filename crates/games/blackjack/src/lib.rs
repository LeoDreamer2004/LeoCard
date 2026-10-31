//! 系统庄家的六副牌 21 点规则。牌序由调用方提供，不依赖网络、时钟或渲染。

mod card;
mod game;
mod profile;
mod rules;
mod strategy;

pub use card::{BlackjackCard, BlackjackHandValue, BlackjackRank, BlackjackSuit, build_shoe};
pub use game::{
    BlackjackAction, BlackjackAvailableActions, BlackjackGameError, BlackjackGameState,
    BlackjackHandResult, BlackjackHandState, BlackjackHandStatus, BlackjackPhase,
    BlackjackPlayerState, BlackjackPlayerStatistics,
};
pub use rules::BlackjackRuleSet;
pub use strategy::{BlackjackAutoPlay, BlackjackBot, BlackjackObservation};

pub use profile::BlackjackProfileStats;
