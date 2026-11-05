mod bot;
mod card;
mod game;
mod match_state;
mod play;
mod profile;
mod rules;

pub use bot::{auto_play_action, hint_options};
pub use card::{GuandanCard, GuandanSuit, build_deck, rank_label};
pub use game::{
    GuandanAction, GuandanActionOutcome, GuandanGameError, GuandanGameResult, GuandanGameState,
    GuandanPhase, GuandanPlayerState, GuandanTributeAction, can_offer_tribute, can_return_tribute,
};
pub use match_state::GuandanMatchState;
pub use play::{GuandanPlay, GuandanPlayError, GuandanPlayKind, can_beat, classify, rank_strength};
pub use profile::GuandanProfileStats;
pub use rules::GuandanRuleSet;
