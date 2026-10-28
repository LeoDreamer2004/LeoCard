//! 与渲染、网络和异步运行时无关的两至四副牌“升级（拖拉机）”规则核心。
//!
//! 随机洗牌和亮主倒计时由宿主负责；核心只接收确定的牌序和玩家操作，因此相同
//! 输入始终得到相同结果，便于联机同步、测试和回放。

pub mod achievements;

mod auto_play;
mod bot;
mod card;
mod game;
mod play;
mod rules;

pub use auto_play::{ShengjiAutoPlay, ShengjiAutoPlayError, ShengjiAutoPlayRequest};

pub use bot::{
    GreedyBotError, ShengjiBiddingObservation, ShengjiBot, ShengjiBotAction, ShengjiBotObservation,
    ShengjiGreedyBot, ShengjiGreedyBotRequest,
};
pub use card::{ShengjiCard, ShengjiRank, ShengjiSuit, build_deck, build_deck_for};
pub use game::{
    ActionOutcome, BidError, BidState, BottomCopyState, BottomFlipMatch, BottomFlipReveal,
    Declaration, FiveTrumpCrossingStage, FiveTrumpCrossingState, GameError, GameState, HandResult,
    Phase, PlayerState, ShengjiBidKind, ShengjiBurialStatistics, ShengjiHandStatistics,
    ShengjiMatchStatistics, ShengjiOpeningHandStatistics, ShengjiRedealReason, TeamProgress,
    TrickRecord, bid_joker_for_suit,
};
pub use play::{
    Category, Component, FollowError, PlayError, ShengjiClassifiedPlay, ThrowFailure, TrickPlay,
    classify_lead, compare_for_trick, follow_suggestions, forced_follow_cards, hand_sequences,
    validate_follow,
};
pub use rules::{
    RuleError, ShengjiBidTrump, ShengjiMode, ShengjiPlayerId, ShengjiRuleSet, ShengjiTeamId,
    ShengjiThrowPenalty, ShengjiTrump, level_after, level_steps_between,
};
