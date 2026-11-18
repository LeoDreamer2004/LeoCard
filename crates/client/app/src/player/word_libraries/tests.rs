use super::LocalWordLibraries;
use leocard_draw_guess::WordLibrary;
use std::env;
use std::fs;

#[test]
fn local_libraries_round_trip_without_overwriting_other_libraries_or_builtin_data() {
    let directory =
        env::temp_dir().join(format!("leocard-word-libraries-{:016x}", fastrand::u64(..)));
    let mut store = LocalWordLibraries::at(Some(directory.clone()));
    assert!(store.reload().unwrap().is_empty());
    let library = WordLibrary::from_lines("动物", "小猫\n小狗\n小熊").unwrap();
    let first = store.save(None, library.clone()).unwrap();
    let second = store.save(None, library).unwrap();
    assert_ne!(first, second);
    store
        .save(
            Some(&first),
            WordLibrary::from_lines("食物", "苹果\n香蕉\n西瓜").unwrap(),
        )
        .unwrap();
    let mut reloaded = LocalWordLibraries::at(Some(directory.clone()));
    assert!(reloaded.reload().unwrap().is_empty());
    assert_eq!(reloaded.entries.len(), 2);
    assert!(
        reloaded
            .entries
            .iter()
            .any(|entry| entry.id == first && entry.library.title == "食物")
    );
    assert!(
        reloaded
            .save(Some("../profile.json"), WordLibrary::builtin())
            .is_err()
    );
    fs::write(directory.join("broken.json"), "invalid").unwrap();
    assert_eq!(reloaded.reload().unwrap().len(), 1);
    assert_eq!(reloaded.entries.len(), 2);
    reloaded.delete(&first).unwrap();
    assert_eq!(reloaded.entries.len(), 1);
    assert_eq!(WordLibrary::builtin().title, "默认词库");
    fs::remove_dir_all(directory).unwrap();
}
