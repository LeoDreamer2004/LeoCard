use super::files::PreferenceFiles;
use super::{GamePreferences, GlobalPreferences, SavedPreferences, migration};
use crate::player::config_file;

fn prepare_files() -> Result<PreferenceFiles, String> {
    let directory = config_file("preferences").ok_or("无法确定本机配置目录")?;
    let files = PreferenceFiles::new(directory);
    migration::migrate(&files)?;
    Ok(files)
}

pub fn load_player_preferences() -> Result<SavedPreferences, String> {
    prepare_files()?.load()
}

pub fn save_global_preferences(preferences: &GlobalPreferences) -> Result<(), String> {
    prepare_files()?.save_global(preferences)
}

pub fn save_game_preferences(preferences: &GamePreferences) -> Result<(), String> {
    prepare_files()?.save_games(preferences)
}
