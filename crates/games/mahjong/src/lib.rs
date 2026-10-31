//! 与渲染、网络和随机数来源无关的 2014 版国标麻将规则核心。
//!
//! 核心使用完整 144 张牌，负责牌墙校验、吃碰杠响应、补花、和牌判定、81 番种
//! 计分及单局至全庄的盘序推进。洗牌和操作倒计时由房主负责。

pub mod achievements;

mod bot;
mod card;
mod game;
mod meld;
mod profile;
mod rules;
mod scoring;

pub use bot::{MahjongBot, MahjongBotAction, MahjongBotObservation};
pub use card::{
    MahjongDragon, MahjongFlower, MahjongSuit, MahjongTile, MahjongTileKind, MahjongWind,
    build_deck,
};
pub use game::{
    MahjongActionOutcome, MahjongClaim, MahjongClaimOption, MahjongDiscard, MahjongDrawOrigin,
    MahjongGameError, MahjongGameState, MahjongHandReplacementError, MahjongHandResult,
    MahjongMatchProgress, MahjongPendingClaim, MahjongPhase, MahjongPlayerState, MahjongPublicMeld,
    MahjongPublicPlayerState, MahjongWinAvailability, MahjongWinRecord,
};
pub use meld::{MahjongKongKind, MahjongMeld, MahjongMeldKind};
pub use rules::{
    MahjongMatchLength, MahjongRuleError, MahjongRuleSet, MahjongUmaStyle, mahjong_reference_deltas,
};
pub use scoring::{
    MahjongFan, MahjongFanValue, MahjongScoreError, MahjongScoreInput, MahjongScoreResult,
    MahjongWinContext, MahjongWinSource, is_complete_hand, score_hand, waiting_tile_kinds,
};

pub use profile::{MAHJONG_MAJOR_FANS, MahjongProfileStats};
