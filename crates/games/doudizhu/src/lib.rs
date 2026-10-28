pub mod achievements;
mod bot;
mod card;
mod game;
mod match_state;
mod play;
mod rules;

pub use bot::{auto_play_action, bot_action, hint, hint_options};
pub use card::{Card, build_deck, rank_label};
pub use game::{Action, GameError, GameState, Phase, ResultView};
pub use match_state::{MatchProgress, MatchState};
pub use play::{Play, PlayKind, classify};
pub use rules::{Mode, RuleSet};

#[cfg(test)]
mod tests;
