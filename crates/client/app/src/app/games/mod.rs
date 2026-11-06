//! 各棋牌游戏独立的客户端状态、输入和表现层。

pub(super) mod blackjack;
mod catalog;
mod composition;
pub(super) mod doudizhu;
mod feature;
pub(super) mod guandan;
pub(super) mod mahjong;
pub(super) mod mahjong_link;
pub(super) mod mc_guess;
mod plugin;
pub(super) mod qigui523;
pub(super) mod shengji;
pub(super) mod texas;
pub(super) mod uno;

pub(super) use catalog::SUPPORTED_GAMES;
pub(super) use composition::GameScreens;
pub(crate) use feature::{
    CardCounter, CounterRows, GameActionPrompt, GameDescriptor, GameFeedback, GameLobbyContext,
    GameLobbyRules, GameMetadata, GameScreen, GameScreenChange, GameSummary, GameTableContext,
    GameUiLifecycle, HandSelection, ScreenChangeCapture, sync_summary_playback,
};
pub(super) use plugin::GamesPlugin;
