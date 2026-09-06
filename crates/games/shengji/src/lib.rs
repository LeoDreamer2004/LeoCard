//! 与渲染、网络和异步运行时无关的两至四副牌“升级（拖拉机）”规则核心。
//!
//! 随机洗牌和亮主倒计时由宿主负责；核心只接收确定的牌序和玩家操作，因此相同
//! 输入始终得到相同结果，便于联机同步、测试和回放。

mod bidding;
mod bot;
mod card;
mod game;
mod play;
mod rules;

pub use bidding::{BidError, BidState, Declaration, ShengjiBidKind, bid_joker_for_suit};
pub use bot::{GreedyBotError, ShengjiGreedyBot, ShengjiGreedyBotRequest};
pub use card::{ShengjiCard, ShengjiRank, ShengjiSuit, build_deck, build_deck_for};
pub use game::{
    ActionOutcome, BottomCopyState, BottomFlipMatch, BottomFlipReveal, FiveTrumpCrossingStage,
    FiveTrumpCrossingState, GameError, GameState, HandResult, Phase, PlayerState, TeamProgress,
    TrickRecord,
};
pub use play::{
    Category, Component, FollowError, PlayError, ShengjiClassifiedPlay, ThrowFailure, TrickPlay,
    classify_lead, compare_for_trick, follow_suggestions, forced_follow_cards, validate_follow,
};
pub use rules::{
    RuleError, ShengjiBidTrump, ShengjiPlayerId, ShengjiRuleSet, ShengjiTeamId,
    ShengjiThrowPenalty, ShengjiTrump, level_after, level_steps_between,
};
