use super::gate::{ActionContext, ActionReminder};
#[cfg(not(target_os = "android"))]
use super::systems::reminder_body;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, MatchId, RoomId};
use std::time::{Duration, Instant};

fn context() -> ActionContext {
    ActionContext {
        room: RoomId(523),
        game: GameKind::Uno,
        match_id: MatchId([1; 16]),
        player: PlayerId(0),
    }
}

#[test]
fn focus_cooldown_and_cancelling_expired_actions() {
    let mut reminder = ActionReminder::default();
    let start = Instant::now();
    let action = Some(context());
    assert!(!reminder.observe(action, true, start));
    assert!(reminder.observe(action, false, start));
    // UI 重建、重复快照和持续等待都不重发。
    assert!(!reminder.observe(action, false, start + Duration::from_secs(60)));
    assert!(!reminder.observe(None, false, start + Duration::from_secs(61)));
    assert!(reminder.observe(action, false, start + Duration::from_secs(62)));
    assert!(!reminder.observe(None, false, start + Duration::from_secs(63)));
    assert!(!reminder.observe(action, false, start + Duration::from_secs(90)));
    // 冷却结束前已轮到别人，不应补发陈旧通知。
    assert!(!reminder.observe(None, false, start + Duration::from_secs(108)));
    assert!(reminder.observe(action, false, start + Duration::from_secs(109)));
}

#[test]
fn cooldown_is_global_across_rooms_and_focus_changes() {
    let mut reminder = ActionReminder::default();
    let start = Instant::now();
    assert!(reminder.observe(Some(context()), false, start));
    let other = Some(ActionContext {
        room: RoomId(524),
        ..context()
    });
    assert!(!reminder.observe(other, false, start + Duration::from_secs(20)));
    assert!(!reminder.observe(other, true, start + Duration::from_secs(45)));
    assert!(reminder.observe(other, false, start + Duration::from_secs(45)));
}

#[cfg(not(target_os = "android"))]
#[test]
fn notification_names_have_one_separator() {
    for (name, expected) in [
        ("Yang", "官儿，轮到你行动了！"),
        ("Snowy", "马眼，轮到你行动了！"),
        ("Dreamer", "Dev，轮到你行动了！"),
        ("ofne", "群主，轮到你行动了！"),
        ("Alice", "Alice，轮到你行动了！"),
    ] {
        assert_eq!(reminder_body(name), expected);
    }
}
