//! 七鬼五二三行动时钟的快照同步与文本刷新。

use super::*;
use leocard_protocol::QiGui523Snapshot;

#[derive(Component)]
pub struct TurnClock;

#[derive(Component)]
pub struct TurnClockHand;

#[derive(Component)]
pub struct TurnClockLabel;

pub fn only_turn_timer_changed(
    before: Option<&QiGui523Snapshot>,
    after: Option<&QiGui523Snapshot>,
) -> bool {
    let (Some(before), Some(after)) = (before, after) else {
        return false;
    };
    if before.turn_timer == after.turn_timer {
        return false;
    }
    let mut normalized = before.clone();
    normalized.turn_timer = after.turn_timer;
    normalized == *after
}

pub fn sync_turn_timer_label(
    client: Option<Res<ClientResource>>,
    mut labels: Query<&mut Text, With<TurnClockLabel>>,
) {
    let timer = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
        .and_then(|game| game.turn_timer);
    let expected = turn_timer_label(timer);
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}
