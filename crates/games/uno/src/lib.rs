//! 与渲染、网络和随机数来源无关的 UNO 规则核心。
//!
//! 牌堆由房主洗牌后传入；核心负责经典 108 张牌的校验、发牌、出牌、罚牌叠加、
//! `+4` 质疑、UNO 宣告/检举，以及最终排名和参考积分计算。

mod card;
mod game;
mod rating;
mod rules;

pub use card::{Card, Color, Face, build_deck};
pub use game::{
    ActionOutcome, ChallengeResult, Direction, GameError, GameResult, GameState, PendingDrawKind,
    Phase, PlayerId, PlayerState, TurnState,
};
pub use rating::reference_point_deltas;
pub use rules::{RuleError, RuleSet};
