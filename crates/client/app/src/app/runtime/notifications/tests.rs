use super::gate::{ActionContext, ActionReminder};
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
fn focused_action_does_not_notify_when_switching_away_later() {
    let mut reminder = ActionReminder::default();
    let start = Instant::now();
    let action = Some(context());
    assert!(!reminder.observe(action, true, start));
    assert!(!reminder.observe(action, false, start + Duration::from_secs(1)));
    assert!(!reminder.observe(action, false, start + Duration::from_secs(60)));
    assert!(!reminder.observe(action, true, start + Duration::from_secs(61)));
    assert!(!reminder.observe(action, false, start + Duration::from_secs(62)));
    // 结束上一次等待后，下一次轮到本人且窗口在后台时才能提醒。
    assert!(!reminder.observe(None, false, start + Duration::from_secs(63)));
    assert!(reminder.observe(action, false, start + Duration::from_secs(64)));
}

#[test]
fn background_action_notifies_once_and_never_catches_up_after_cooldown() {
    let mut reminder = ActionReminder::default();
    let start = Instant::now();
    let action = Some(context());
    assert!(reminder.observe(action, false, start));
    // UI 重建、重复快照和持续等待都不重发。
    assert!(!reminder.observe(action, false, start + Duration::from_secs(60)));
    assert!(!reminder.observe(None, false, start + Duration::from_secs(61)));
    assert!(reminder.observe(action, false, start + Duration::from_secs(62)));
    assert!(!reminder.observe(None, false, start + Duration::from_secs(63)));
    assert!(!reminder.observe(action, false, start + Duration::from_secs(90)));
    // 冷却期间开始的等待也只检查一次，冷却结束后不补发。
    assert!(!reminder.observe(action, false, start + Duration::from_secs(108)));
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
    assert!(!reminder.observe(other, false, start + Duration::from_secs(45)));
    assert!(!reminder.observe(None, false, start + Duration::from_secs(46)));
    assert!(reminder.observe(other, false, start + Duration::from_secs(47)));
}
