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
mod profile;
mod rules;

pub use auto_play::{ShengjiAutoPlay, ShengjiAutoPlayError, ShengjiAutoPlayRequest};

pub use bot::{
    ShengjiBiddingObservation, ShengjiBot, ShengjiBotAction, ShengjiBotObservation,
    ShengjiGreedyBot, ShengjiGreedyBotError, ShengjiGreedyBotRequest,
};
pub use card::{ShengjiCard, ShengjiRank, ShengjiSuit, build_deck, build_deck_for};
pub use game::{
    ShengjiActionOutcome, ShengjiBidError, ShengjiBidKind, ShengjiBidState, ShengjiBottomCopyState,
    ShengjiBottomFlipMatch, ShengjiBottomFlipReveal, ShengjiBurialStatistics, ShengjiDeclaration,
    ShengjiFiveTrumpCrossingStage, ShengjiFiveTrumpCrossingState, ShengjiGameError,
    ShengjiGameState, ShengjiHandResult, ShengjiHandStatistics, ShengjiMatchStatistics,
    ShengjiOpeningHandStatistics, ShengjiPhase, ShengjiPlayerState, ShengjiRedealReason,
    ShengjiTeamProgress, ShengjiTrickRecord, bid_joker_for_suit,
};
pub use play::{
    ShengjiCategory, ShengjiClassifiedPlay, ShengjiComponent, ShengjiFollowError, ShengjiPlayError,
    ShengjiThrowFailure, ShengjiTrickPlay, classify_lead, compare_for_trick, follow_suggestions,
    forced_follow_cards, hand_sequences, validate_follow,
};
pub use rules::{
    ShengjiBidTrump, ShengjiMode, ShengjiRuleError, ShengjiRuleSet, ShengjiTeamId,
    ShengjiThrowPenalty, ShengjiTrump, level_after, level_steps_between,
};

pub use profile::ShengjiProfileStats;
