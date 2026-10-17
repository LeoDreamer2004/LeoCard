//! 系统庄家的六副牌 21 点规则。牌序由调用方提供，不依赖网络、时钟或渲染。

mod card;
mod game;
mod rules;
mod strategy;

pub use card::{BlackjackCard, BlackjackRank, BlackjackSuit, HandValue, build_shoe};
pub use game::{
    AvailableActions, BlackjackAction, GameError, GameState, HandResult, HandState, HandStatus,
    Phase, PlayerState, PlayerStatistics,
};
pub use rules::BlackjackRuleSet;
pub use strategy::{BlackjackAutoPlay, BlackjackBot, BlackjackObservation};
