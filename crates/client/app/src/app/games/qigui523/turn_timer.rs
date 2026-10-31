//! 七鬼五二三行动时钟的快照同步与文本刷新。
use leocard_protocol::{GameSnapshot, QiGui523Snapshot};

use super::turn_timer_label;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct TurnClockLabel;

pub(crate) fn only_turn_timer_changed(
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

pub(super) fn sync_turn_timer_label(
    client: Option<Res<ClientResource>>,
    mut labels: Query<&mut Text, With<TurnClockLabel>>,
) {
    let timer = client
        .as_deref()
        .and_then(|client| {
            client
                .0
                .model()
                .game_snapshot()
                .and_then(GameSnapshot::qigui523)
        })
        .and_then(|game| game.turn_timer);
    let expected = turn_timer_label(timer);
    for mut label in &mut labels {
        if label.0 != expected {
            label.0.clone_from(&expected);
        }
    }
}
