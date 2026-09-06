//! 与渲染、网络和随机数来源无关的 2014 版国标麻将规则核心。
//!
//! 核心使用完整 144 张牌，负责牌墙校验、吃碰杠响应、补花、和牌判定、81 番种
//! 计分及单局至全庄的盘序推进。洗牌和操作倒计时由房主负责。

mod card;
mod game;
mod meld;
mod rules;
mod scoring;

pub use card::{
    MahjongDragon, MahjongFlower, MahjongSuit, MahjongTile, MahjongTileKind, MahjongWind,
    build_deck,
};
pub use game::{
    ActionOutcome, Discard, GameError, GameState, HandResult, MahjongClaim, MahjongClaimOption,
    MahjongDrawOrigin, PendingClaim, Phase, PlayerState, PublicMeld, PublicPlayerState, WinRecord,
};
pub use meld::{MahjongKongKind, MahjongMeldKind, Meld};
pub use rules::{MahjongMatchLength, MahjongPlayerId, MahjongRuleSet, RuleError};
pub use scoring::{
    Fan, FanValue, MahjongScoreResult, ScoreError, ScoreInput, WinContext, WinSource,
    is_complete_hand, score_hand,
};
