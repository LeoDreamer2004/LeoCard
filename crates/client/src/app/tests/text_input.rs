use super::*;
use leocard_protocol::MAX_PLAYER_NAME_CHARS;

#[test]
fn saved_player_name_is_limited_by_unicode_characters() {
    let name = truncate_chars("一二三四五六七八", MAX_PLAYER_NAME_CHARS);
    assert_eq!(name, "一二三四五六七");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn ime_committed_chinese_is_accepted_by_the_player_name_filter() {
    let mut name = String::new();
    append_filtered_input(
        &mut name,
        InputField::PlayerName,
        "七鬼五二三玩家甲",
        MAX_PLAYER_NAME_CHARS,
    );

    assert_eq!(name, "七鬼五二三玩家");
    assert_eq!(name.chars().count(), MAX_PLAYER_NAME_CHARS);
}

#[test]
fn pasted_server_address_filters_whitespace_and_replaces_the_default_value() {
    let mut address = String::new();
    append_filtered_input(
        &mut address,
        InputField::JoinAddress,
        " 192.168.1.20:52300\n",
        64,
    );

    assert_eq!(address, "192.168.1.20:52300");

    let mut hostname = String::new();
    append_filtered_input(
        &mut hostname,
        InputField::JoinAddress,
        "frp-off.com:52436",
        64,
    );
    assert_eq!(hostname, "frp-off.com:52436");
}
