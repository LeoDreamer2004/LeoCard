mod files;
mod games;
mod global;
mod migration;
mod state;
mod storage;

pub use games::{
    GamePreferences, MahjongPreferences, MinecraftGuessPreferences, QiGui523Preferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
pub use global::GlobalPreferences;
pub use state::SavedPreferences;
pub use storage::{load_player_preferences, save_game_preferences, save_global_preferences};

#[cfg(test)]
mod tests;
