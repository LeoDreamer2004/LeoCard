pub mod achievements;
mod bot;
mod card;
mod douzero;
mod four_player;
mod game;
mod match_state;
mod play;
mod profile;
mod rules;

pub use bot::{auto_play_action, bot_action, hint, hint_options};
pub use card::{DoudizhuCard, build_deck, rank_label};
pub use douzero::{DouzeroBot, DouzeroObservation};
pub use four_player::{FourPlayerBot, FourPlayerObservation};
pub use game::{
    DoudizhuAction, DoudizhuGameError, DoudizhuGameState, DoudizhuPhase, DoudizhuResultView,
};
pub use match_state::{DoudizhuMatchProgress, DoudizhuMatchState};
pub use play::{DoudizhuPlay, DoudizhuPlayKind, classify};
pub use rules::{DoudizhuMode, DoudizhuRuleSet};

pub use profile::DoudizhuProfileStats;
