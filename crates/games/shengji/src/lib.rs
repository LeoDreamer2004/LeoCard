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

pub use bidding::{BidError, BidKind, BidState, Declaration, bid_joker_for_suit};
pub use bot::{GreedyBot, GreedyBotError, GreedyBotRequest};
pub use card::{Card, Rank, Suit, build_deck, build_deck_for};
pub use game::{
    ActionOutcome, BottomCopyState, BottomFlipMatch, BottomFlipReveal, FiveTrumpCrossingStage,
    FiveTrumpCrossingState, GameError, GameState, HandResult, Phase, PlayerState, TeamProgress,
    TrickRecord,
};
pub use play::{
    Category, ClassifiedPlay, Component, FollowError, PlayError, ThrowFailure, TrickPlay,
    classify_lead, compare_for_trick, follow_suggestions, forced_follow_cards, validate_follow,
};
pub use rules::{
    BidTrump, PlayerId, RuleError, RuleSet, TeamId, ThrowPenalty, Trump, level_after,
    level_steps_between,
};
