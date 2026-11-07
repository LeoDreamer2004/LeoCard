use super::crypto::encrypt_request;
use super::{LoginPoll, MusicClient, QrLogin, Song};
use aes::Aes128;
use ecb::Decryptor;
use ecb::cipher::{BlockDecryptMut, KeyInit, block_padding::Pkcs7};
use std::{env, fs};

#[test]
fn request_envelope_contains_route_payload_and_protocol_digest() {
    let encrypted = encrypt_request("/api/test", r#"{"id":1}"#);
    let bytes = encrypted
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let plain = Decryptor::<Aes128>::new(b"e82ckenh8dichen8".into())
        .decrypt_padded_vec_mut::<Pkcs7>(&bytes)
        .unwrap();
    assert_eq!(
        String::from_utf8(plain).unwrap(),
        "/api/test-36cd479b6b5-{\"id\":1}-36cd479b6b5-1ecf2a6fa7bab4cce1ccbaa20c81d451"
    );
}

#[test]
fn qr_pixels_keep_the_quiet_zone_opaque_white() {
    let qr = QrLogin {
        key: "test".to_owned(),
        url: "https://music.163.com/login?codekey=test".to_owned(),
    };
    let (pixels, width) = qr.pixels().unwrap();
    assert_eq!(pixels.len(), width as usize * width as usize * 4);
    assert!(
        pixels[..width as usize * 4 * 20]
            .iter()
            .all(|value| *value == 255)
    );
    assert!(
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|pixel| pixel[3] == 255)
    );
    assert!(pixels.as_chunks::<4>().0.contains(&[0, 0, 0, 255]));
}

#[test]
fn song_metadata_preserves_multiple_artists() {
    let song: Song = serde_json::from_str(
        r#"{"id":1,"name":"歌曲","ar":[{"name":"甲"},{"name":"乙"}],"dt":120000}"#,
    )
    .unwrap();
    assert_eq!(song.artist_names(), "甲 / 乙");
    assert_eq!(song.duration_ms, 120000);
}

#[test]
fn playlist_keeps_songs_whose_artist_name_is_null() {
    let songs: Vec<Song> = serde_json::from_str(
        r#"[
        {"id":1,"name":"正常曲目","ar":[{"name":"歌手"}],"dt":120000},
        {"id":2,"name":"歌手资料已失效的曲目","ar":[{"name":null}],"dt":120000}
    ]"#,
    )
    .expect("one null artist name must not prevent the whole playlist from loading");
    assert_eq!(songs.len(), 2);
    assert_eq!(songs[0].artist_names(), "歌手");
    assert_eq!(songs[1].artist_names(), "未知歌手");
}

#[test]
#[ignore = "contacts NetEase directly; no credentials or user account required"]
fn netease_live_qr_and_public_catalog() {
    let client = MusicClient::new().unwrap();
    let qr = client.qr_login().unwrap();
    assert!(!qr.key.is_empty());
    assert!(matches!(
        client.poll_login(&qr.key).unwrap(),
        LoginPoll::Waiting
    ));
    // NetEase's public rising chart exercises playlist detail and paginated song metadata.
    let songs = client.songs(19723756).unwrap();
    assert!(!songs.is_empty());
    assert!(
        songs
            .iter()
            .all(|song| song.id > 0 && !song.name.is_empty())
    );
    client.playlists(32953014).unwrap();
}

#[test]
#[ignore = "downloads a publicly playable song from NetEase; no account required"]
fn netease_live_audio_download() {
    let audio = MusicClient::new().unwrap().audio(33894312).unwrap();
    assert!(audio.bytes.len() > 1024);
}

#[test]
#[ignore = "uses the local account selected by LEOCARD_MUSIC_SESSION_FILE"]
fn netease_saved_account_liked_songs() {
    let path = env::var_os("LEOCARD_MUSIC_SESSION_FILE")
        .expect("set LEOCARD_MUSIC_SESSION_FILE to the local music configuration");
    let preferences: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let session = serde_json::from_value(preferences["session"].clone()).unwrap();
    let client = MusicClient::restore(&session).unwrap();
    let playlists = client.playlists(session.account.id).unwrap();
    let liked = playlists
        .iter()
        .find(|playlist| {
            playlist.creator.id == session.account.id && playlist.name == "我喜欢的音乐"
        })
        .expect("account has a liked-songs playlist");
    let songs = client
        .songs(liked.id)
        .unwrap_or_else(|error| panic!("读取已关联账号歌单失败：{error}"));
    assert!(!songs.is_empty());
}
