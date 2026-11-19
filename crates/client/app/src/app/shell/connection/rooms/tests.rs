use super::store::SavedRoomStore;
use image::{ImageFormat, Rgba, RgbaImage};
use leocard_protocol::{AVATAR_DIMENSION, AvatarId, GameKind, RoomInfo};
use std::env;
use std::fs;
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};

fn info(color: [u8; 4]) -> RoomInfo {
    let mut png = Cursor::new(Vec::new());
    RgbaImage::from_pixel(AVATAR_DIMENSION, AVATAR_DIMENSION, Rgba(color))
        .write_to(&mut png, ImageFormat::Png)
        .unwrap();
    RoomInfo {
        host_name: "房主".to_owned(),
        host_avatar: Some(AvatarId(1)),
        avatar_png: Some(png.into_inner()),
        game: GameKind::Mahjong,
        playing: false,
    }
}

#[test]
fn saved_rooms_keep_custom_names_and_separate_server_local_avatar_ids() {
    let directory = env::temp_dir().join(format!(
        "leocard-rooms-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = directory.join("rooms.json");
    let mut store = SavedRoomStore::load(Some(path.clone()));
    let first = "127.0.0.1:52300";
    let second = "localhost:52301";
    store.save(None, "常用房间", first).unwrap();
    let red = info([220, 30, 30, 255]);
    let blue = info([30, 30, 220, 255]);
    store.remember(first, &red).unwrap();
    store.remember(second, &blue).unwrap();
    let mut store = SavedRoomStore::load(Some(path.clone()));
    assert_eq!(store.entries.len(), 2);
    assert_eq!(store.entries[0].name, "常用房间");
    let red_key = store.entries[0]
        .host
        .as_ref()
        .unwrap()
        .image
        .as_ref()
        .unwrap();
    let blue_key = store.entries[1]
        .host
        .as_ref()
        .unwrap()
        .image
        .as_ref()
        .unwrap();
    assert_ne!(red_key, blue_key);
    assert_eq!(store.avatar(red_key).as_ref(), red.avatar_png.as_ref());
    assert_eq!(store.avatar(blue_key).as_ref(), blue.avatar_png.as_ref());
    let cached = store.entries[0].host.clone();
    let mut broadcast = red;
    broadcast.avatar_png = None;
    assert!(!store.remember(first, &broadcast).unwrap());
    assert_eq!(store.entries[0].host, cached);
    store.save(Some(first), "改名", first).unwrap();
    assert_eq!(store.entries[0].host, cached);
    assert!(store.save(None, "重复地址", first).is_err());
    assert!(store.save(Some(first), "重复地址", second).is_err());
    assert_eq!(store.entries.len(), 2);
    store
        .save(Some(first), "新地址", "127.0.0.1:52302")
        .unwrap();
    assert!(store.entries[0].host.is_none());
    store.remove(second).unwrap();
    assert_eq!(SavedRoomStore::load(Some(path)).entries.len(), 1);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invalid_room_addresses_do_not_create_a_saved_entry() {
    let mut store = SavedRoomStore::load(None);
    for address in ["", "localhost", "localhost:0", "localhost:not-a-port"] {
        assert!(store.save(None, "房间", address).is_err());
    }
    assert!(store.entries.is_empty());
    assert!(store.avatar("../unrelated").is_none());
}
