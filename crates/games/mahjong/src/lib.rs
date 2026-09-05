//! 与渲染、网络和随机数来源无关的 2014 版国标麻将规则核心。
//!
//! 核心使用完整 144 张牌，负责牌墙校验、吃碰杠响应、补花、和牌判定、81 番种
//! 计分及单局至全庄的盘序推进。洗牌和操作倒计时由房主负责。

mod card;
mod game;
mod meld;
mod rules;
mod scoring;

pub use card::{Dragon, Flower, Suit, Tile, TileKind, Wind, build_deck};
pub use game::{
    ActionOutcome, Claim, ClaimOption, Discard, DrawOrigin, GameError, GameState, HandResult,
    PendingClaim, Phase, PlayerState, PublicMeld, PublicPlayerState, WinRecord,
};
pub use meld::{KongKind, Meld, MeldKind};
pub use rules::{MatchLength, PlayerId, RuleError, RuleSet};
pub use scoring::{
    Fan, FanValue, ScoreError, ScoreInput, ScoreResult, WinContext, WinSource, is_complete_hand,
    score_hand,
};
