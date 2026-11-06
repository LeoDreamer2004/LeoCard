//! 七鬼五二三行动时钟的快照同步与文本刷新。

use super::turn_timer_label;
use crate::app::games::GameScreenChange;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use leocard_protocol::{GameSnapshot, QiGui523Snapshot};

#[derive(Component)]
pub(super) struct TurnClockLabel;

impl GameScreenChange for QiGui523Snapshot {
    fn is_transient_change(&self, after: &Self) -> bool {
        if self.turn_timer == after.turn_timer {
            return false;
        }
        let mut normalized = self.clone();
        normalized.turn_timer = after.turn_timer;
        normalized == *after
    }
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
