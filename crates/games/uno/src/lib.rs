//! 与渲染、网络和随机数来源无关的 UNO 规则核心。
//!
//! 牌堆由房主洗牌后传入；核心负责经典 108 张牌的校验、发牌、出牌、罚牌叠加、
//! `+4` 质疑、UNO 宣告/检举，以及最终排名和参考积分计算。

mod card;
mod game;
mod rating;
mod rules;

pub use card::{
    Card, CardSide, Color, Face, FlipSide, build_deck, build_deck_for_rules, build_flip_dark_sides,
    build_flip_deck, build_flip_light_sides, build_no_mercy_deck, build_reverse_pack,
    build_stack_pack, build_swap_pack, pair_flip_deck,
};
pub use game::{
    ActionOutcome, ChallengeResult, Direction, GameError, GameResult, GameState, PendingDrawKind,
    PendingSwap, Phase, PlayedEffect, PlayerId, PlayerState, TurnState,
};
pub use rating::reference_point_deltas;
pub use rules::{FlipRuleSet, Mode, NoMercyRuleSet, RuleError, RuleSet};
