//! 与渲染、网络和异步运行时无关的德州扑克规则核心。
//!
//! 牌堆由房主洗牌后传入；相同牌序和动作始终产生相同结果，便于联机同步、测试与回放。

mod bot;
mod card;
mod game;
mod hand;
mod rules;

pub use bot::{PassiveBot, PassiveBotRequest};
pub use card::{TexasHoldemCard, TexasHoldemRank, TexasHoldemSuit, build_deck};
pub use game::{
    ActionOutcome, GameError, GameState, HandResult, Phase, PlayerState, PotAward,
    TexasHoldemAction, TexasHoldemBlindKind, TexasHoldemPlayerId, TexasHoldemStreet,
};
pub use hand::{
    EvaluatedHand, HandError, TexasHoldemHandCategory, evaluate_best, evaluate_omaha,
    evaluate_player_hand,
};
pub use rules::{RuleError, TexasHoldemRuleSet};
