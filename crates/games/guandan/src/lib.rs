mod bot;
mod card;
mod game;
mod match_state;
mod play;
mod rules;

pub use bot::auto_play_action;
pub use card::{GuandanCard, GuandanSuit, build_deck, rank_label};
pub use game::{
    GuandanAction, GuandanActionOutcome, GuandanGameError, GuandanGameResult, GuandanGameState,
    GuandanPhase, GuandanPlayerState, GuandanTributeAction,
};
pub use match_state::GuandanMatchState;
pub use play::{GuandanPlay, GuandanPlayError, GuandanPlayKind, can_beat, classify};
pub use rules::GuandanRuleSet;
