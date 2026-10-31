use crate::app::runtime::{ClientResource, ServerNotification};
use crate::app::shell::{PlayErrorToast, UiState};
use bevy::prelude::*;
use leocard_protocol::{GameEvent, GameSnapshot, MatchId, McGuessEvent, ServerEvent};

use std::collections::VecDeque;

#[derive(Resource, Default)]
pub(super) struct McProcessNotices {
    match_id: Option<MatchId>,
    pending: VecDeque<String>,
}

pub(super) fn show_process_notices(
    client: Option<Res<ClientResource>>,
    mut events: MessageReader<ServerNotification>,
    mut notices: ResMut<McProcessNotices>,
    mut toast: ResMut<PlayErrorToast>,
    mut ui: ResMut<UiState>,
) {
    let match_id = client
        .as_deref()
        .and_then(|client| {
            client
                .0
                .model()
                .game_snapshot()
                .and_then(GameSnapshot::mc_guess)
        })
        .map(|game| game.match_id);
    if notices.match_id != match_id {
        notices.match_id = match_id;
        notices.pending.clear();
    }
    for notification in events.read() {
        let ServerEvent::GameEvent(GameEvent::McGuess(McGuessEvent::ProcessPresenceChanged {
            player,
            name,
            running,
        })) = &notification.event
        else {
            continue;
        };
        if match_id.is_none()
            || notification.game_context.map(|context| context.match_id) != match_id
        {
            continue;
        }
        let name = if notification.player == Some(*player) {
            "你"
        } else {
            name
        };
        notices.pending.push_back(if *running {
            format!("检测到{name}后台有 Minecraft 进程！")
        } else {
            format!("检测到{name}后台关闭了 Minecraft 进程")
        });
    }
    if !toast.active
        && let Some(message) = notices.pending.pop_front()
    {
        toast.show_message(message);
        ui.dirty = true;
    }
}
