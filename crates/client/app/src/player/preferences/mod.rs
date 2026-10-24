mod games;
mod global;
mod state;
mod storage;

pub use games::{
    GamePreferences, MahjongPreferences, MinecraftGuessPreferences, QiGui523Preferences,
    ShengjiPreferences, TexasHoldemPreferences, UnoPreferences,
};
pub use global::GlobalPreferences;
pub use state::SavedPreferences;
pub use storage::{load_player_preferences, save_player_preferences};
