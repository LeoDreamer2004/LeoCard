#[cfg(not(target_arch = "wasm32"))]
mod ai;
mod appearance;
mod controls;
mod files;
mod games;
mod global;
mod migration;
mod state;
mod storage;

#[cfg(not(target_arch = "wasm32"))]
pub use ai::AiPreferences;
pub use appearance::GameAppearancePreferences;
pub use controls::ControlPreferences;
pub use games::{
    DoudizhuPreferences, GamePreferences, GuandanPreferences, MahjongPreferences,
    McGuessPreferences, QiGui523Preferences, ShengjiPreferences, TexasPreferences, UnoPreferences,
};
pub use global::GlobalPreferences;
pub use state::SavedPreferences;
pub use storage::{load_player_preferences, save_game_preferences, save_global_preferences};

#[cfg(test)]
mod tests;
