//! 与渲染、网络和异步运行时无关的“七鬼五二三”规则核心。
//!
//! 牌堆应由房主洗牌，然后通过 [`GameState::new_with_deck`] 传入。规则核心不自行
//! 产生随机数，因此相同的牌序与玩家操作一定会得到相同结果，便于联网同步、回放
//! 和测试。

mod card;
mod game;
mod greedy;
mod play;
mod rating;
mod rules;

pub use card::{Card, Rank, Suit, build_deck};
pub use game::{
    ActionOutcome, GameError, GameResult, GameState, Phase, PlayRecord, PlayerId, PlayerState,
    StartingCard, TrickState,
};
pub use greedy::{GreedyRequest, GreedyStrategy, has_legal_response};
pub use play::{
    BombKind, ClassifiedPlay, PlayComparison, PlayError, PlayKind, can_beat, classify,
    compare_plays,
};
pub use rating::reference_point_deltas;
pub use rules::{RuleError, RuleSet, SameCardPolicy, SuitComparison, TimeControl};
