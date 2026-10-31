//! 与渲染、网络和异步运行时无关的“七鬼五二三”规则核心。
//!
//! 牌堆应由房主洗牌，然后通过 [`QiGuiGameState::new_with_deck`] 传入。规则核心不自行
//! 产生随机数，因此相同的牌序与玩家操作一定会得到相同结果，便于联网同步、回放
//! 和测试。

pub mod achievements;

mod bot;
mod card;
mod game;
mod play;
mod profile;
mod rating;
mod rules;
mod statistics;

pub use bot::{QiGui523Bot, QiGui523BotObservation, QiGui523BotRequest, has_legal_response};
pub use card::{QiGuiCard, QiGuiRank, QiGuiSuit, build_deck};
pub use game::{
    QiGuiActionOutcome, QiGuiGameError, QiGuiGameResult, QiGuiGameState, QiGuiPhase,
    QiGuiPlayRecord, QiGuiPlayerState, QiGuiStartingCard, QiGuiTrickState,
};
pub use play::{
    QiGuiBombKind, QiGuiClassifiedPlay, QiGuiPlayComparison, QiGuiPlayError, QiGuiPlayKind,
    can_beat, classify, compare_plays,
};
pub use rating::reference_point_deltas;
pub use rules::{
    QiGuiRuleError, QiGuiRuleSet, QiGuiSameCardPolicy, QiGuiSuitComparison, QiGuiTimeControl,
};

pub use statistics::{
    QiGuiActionContext, QiGuiActionStatistics, QiGuiMatchStatistics, QiGuiPlayerStatistics,
};

pub use profile::QiGui523ProfileStats;
