//! 与渲染、网络和异步运行时无关的德州扑克规则核心。
//!
//! 牌堆由房主洗牌后传入；相同牌序和动作始终产生相同结果，便于联机同步、测试与回放。

mod auto_play;
mod bot;
mod card;
mod game;
mod hand;
mod rating;
mod rules;
mod statistics;
pub use rating::reference_point_deltas;
pub use statistics::{
    TexasHoldemActionStatistics, TexasHoldemHandStatistics, TexasHoldemMatchStatistics,
};

pub use auto_play::{PassiveAutoPlay, PassiveAutoPlayRequest};

pub use bot::{TexasHoldemBot, TexasHoldemBotObservation};
pub use card::{TexasHoldemCard, TexasHoldemRank, TexasHoldemSuit, build_deck};
pub use game::{
    ActionOutcome, BettingRecord, GameError, GameState, HandResult, Phase, PlayerState, PotAward,
    RaiseSizing, TexasHoldemAction, TexasHoldemBlindKind, TexasHoldemPlayerId, TexasHoldemStreet,
};
pub use hand::{
    EvaluatedHand, HandError, TexasHoldemHandCategory, evaluate_best, evaluate_omaha,
    evaluate_player_hand,
};
pub use rules::{RuleError, TexasHoldemRuleSet};
