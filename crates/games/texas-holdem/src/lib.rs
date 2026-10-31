//! 与渲染、网络和异步运行时无关的德州扑克规则核心。
//!
//! 牌堆由房主洗牌后传入；相同牌序和动作始终产生相同结果，便于联机同步、测试与回放。

pub mod achievements;

mod auto_play;
mod bot;
mod card;
mod game;
mod hand;
mod profile;
mod rating;
mod rules;
mod squid;
mod statistics;
pub use rating::reference_point_deltas;
pub use squid::{
    DEEPSEEK_SQUID_VARIANTS, TexasSquidAwardStatistics, TexasSquidState, TexasSquidTransfer,
    deepseek_squid_marker,
};
pub use statistics::{TexasActionStatistics, TexasHandStatistics, TexasMatchStatistics};

pub use auto_play::{TexasPassiveAutoPlay, TexasPassiveAutoPlayRequest};

pub use bot::{TexasBot, TexasBotObservation};
pub use card::{TexasCard, TexasRank, TexasSuit, build_deck};
pub use game::{
    TexasAction, TexasActionOutcome, TexasBettingRecord, TexasBlindKind, TexasGameError,
    TexasGameState, TexasHandResult, TexasPhase, TexasPlayerState, TexasPotAward, TexasRaiseSizing,
    TexasStreet,
};
pub use hand::{
    TexasEvaluatedHand, TexasHandCategory, TexasHandError, evaluate_best, evaluate_omaha,
    evaluate_player_hand,
};
pub use rules::{TexasRuleError, TexasRuleSet};

pub use profile::TexasProfileStats;
