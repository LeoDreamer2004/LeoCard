use super::files::PreferenceFiles;
use super::{GlobalPreferences, SavedPreferences, migration};
use std::env;
use std::fs;
use std::path::PathBuf;

struct TestFiles {
    root: PathBuf,
    files: PreferenceFiles,
}

impl TestFiles {
    fn new() -> Self {
        let root = env::temp_dir().join(format!("leocard-preferences-{:016x}", fastrand::u64(..)));
        fs::create_dir_all(&root).unwrap();
        Self {
            files: PreferenceFiles::new(root.join("preferences")),
            root,
        }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join("preferences").join(name)
    }
}

impl Drop for TestFiles {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn saved_preferences() -> SavedPreferences {
    let mut preferences = SavedPreferences::default();
    preferences.global.player_name = "保留的玩家".to_owned();
    preferences.global.avatar_png = Some(vec![1, 2, 3, 4]);
    preferences.global.audio_volume = 0.37;
    preferences.games.texas_holdem.host_rules.starting_chips = 50;
    preferences.games.texas_holdem.host_rules.squid_markers = 2;
    preferences.games.texas_holdem.host_rules.omaha = true;
    preferences.games.texas_holdem.host_rules.fixed_limit = true;
    preferences.games.shengji.host_rules.bottom_copy = true;
    preferences
        .games
        .minecraft_guess
        .host_rules
        .graph
        .temperature = 8;
    preferences
}

fn legacy_bytes(version: u8, preferences: &SavedPreferences) -> Vec<u8> {
    let mut bytes = format!("LeoCard/prefs/v{version}\0").into_bytes();
    bytes.extend(postcard::to_allocvec(&preferences.global).unwrap());
    if version == 4 {
        let rules = preferences.games.texas_holdem.host_rules;
        bytes.extend(
            postcard::to_allocvec(&(
                &preferences.games.qigui523,
                ((
                    rules.player_count,
                    rules.starting_chips,
                    rules.squid_markers,
                    rules.short_deck,
                    rules.ignore_kickers,
                    rules.omaha,
                    rules.fixed_limit,
                ),),
                &preferences.games.shengji,
                &preferences.games.uno,
                &preferences.games.mahjong,
                &preferences.games.minecraft_guess,
            ))
            .unwrap(),
        );
    } else {
        bytes.extend(postcard::to_allocvec(&preferences.games).unwrap());
    }
    bytes
}

#[test]
fn migration_persists_both_legacy_versions_and_needs_no_legacy_reader_afterwards() {
    for version in [4, 5] {
        let directory = TestFiles::new();
        let mut saved = saved_preferences();
        saved.games.texas_holdem.host_rules.deepseek_squid_double = version == 5;
        let bytes = legacy_bytes(version, &saved);
        fs::write(directory.files.legacy_path(), &bytes).unwrap();
        for name in ["profile.dat", "achievements.dat", "economy.dat"] {
            fs::write(directory.root.join(name), b"untouched").unwrap();
        }
        migration::migrate(&directory.files).unwrap();
        assert!(!directory.files.legacy_path().exists());
        let backup = directory.root.join("client.prefs.migrated");
        assert_eq!(fs::read(&backup).unwrap(), bytes);
        fs::remove_file(backup).unwrap();
        // Read the new files directly, without invoking any migration code.
        let restored = directory.files.load().unwrap();
        if version == 4 {
            saved.games.texas_holdem = SavedPreferences::default().games.texas_holdem;
        }
        assert_eq!(
            serde_json::to_value(&restored).unwrap(),
            serde_json::to_value(saved).unwrap()
        );
        for name in ["profile.dat", "achievements.dat", "economy.dat"] {
            assert_eq!(fs::read(directory.root.join(name)).unwrap(), b"untouched");
        }
    }
}

#[test]
fn invalid_game_rules_reset_only_that_game_and_rule_saves_leave_global_bytes_unchanged() {
    let directory = TestFiles::new();
    let saved = saved_preferences();
    directory.files.import(&saved).unwrap();
    let global = fs::read(directory.path("global.json")).unwrap();
    fs::write(
        directory.path("games/texas_holdem.json"),
        b"{\"host_rules\":{}}",
    )
    .unwrap();
    let restored = directory.files.load().unwrap();
    assert_eq!(restored.global.avatar_png, saved.global.avatar_png);
    assert_eq!(
        restored.games.texas_holdem.host_rules,
        SavedPreferences::default().games.texas_holdem.host_rules
    );
    assert_eq!(
        restored.games.shengji.host_rules,
        saved.games.shengji.host_rules
    );
    directory.files.save_games(&restored.games).unwrap();
    assert_eq!(fs::read(directory.path("global.json")).unwrap(), global);
}

#[test]
fn interrupted_migration_resumes_without_overwriting_already_saved_preferences() {
    let directory = TestFiles::new();
    let saved = saved_preferences();
    fs::write(directory.files.legacy_path(), legacy_bytes(4, &saved)).unwrap();
    fs::create_dir_all(directory.path("games/uno.json")).unwrap();
    assert!(migration::migrate(&directory.files).is_err());
    assert!(directory.files.legacy_path().exists());
    let mut updated_global = saved.global;
    updated_global.player_name = "新设置".to_owned();
    directory.files.save_global(&updated_global).unwrap();
    fs::remove_dir(directory.path("games/uno.json")).unwrap();
    migration::migrate(&directory.files).unwrap();
    let restored = directory.files.load().unwrap();
    assert_eq!(restored.global.player_name, "新设置");
    assert!(restored.games.shengji.host_rules.bottom_copy);
    assert!(!directory.files.legacy_path().exists());
}

#[test]
fn undecodable_legacy_rules_still_migrate_avatar_and_global_settings() {
    let directory = TestFiles::new();
    let saved = saved_preferences();
    let mut bytes = b"LeoCard/prefs/v4\0".to_vec();
    bytes.extend(postcard::to_allocvec(&saved.global).unwrap());
    bytes.extend([255, 255]);
    fs::write(directory.files.legacy_path(), bytes).unwrap();
    migration::migrate(&directory.files).unwrap();
    let restored = directory.files.load().unwrap();
    assert_eq!(
        serde_json::to_value(restored.global).unwrap(),
        serde_json::to_value(saved.global).unwrap()
    );
}

#[test]
fn global_fields_default_independently_without_touching_existing_values() {
    let saved: GlobalPreferences =
        serde_json::from_str(r#"{"player_name":"原玩家","avatar_png":[1,2]}"#).unwrap();
    assert_eq!(saved.player_name, "原玩家");
    assert_eq!(saved.avatar_png, Some(vec![1, 2]));
    assert_eq!(
        saved.audio_volume,
        GlobalPreferences::default().audio_volume
    );
}
