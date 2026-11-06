use super::PendingAction;
use crate::lifecycle::HostedGameLifecycle;
use crate::{Delivery, RoomSession};
use leocard_protocol::{GameKind, MatchId, ServerEvent};

pub(crate) fn sync_action_timing(
    room: &mut RoomSession,
    game: GameKind,
    match_id: MatchId,
    mut windows: Vec<PendingAction>,
) {
    windows.retain(|window| {
        room.players.iter().any(|player| {
            player.id == window.player
                && player.connected
                && !player.left
                && !player.is_bot
                && !player.auto_play
        })
    });
    room.action_timing.sync(game, match_id, windows);
}

pub(crate) fn attach_action_timing<S: HostedGameLifecycle>(
    session: &S,
    mut deliveries: Vec<Delivery>,
) -> Vec<Delivery> {
    let timing = session.room().action_timing.snapshot();
    for delivery in &mut deliveries {
        delivery.message.action_timing.clone_from(&timing);
        if session.game_started()
            && let ServerEvent::GameSnapshot(snapshot) = &delivery.message.event
        {
            // Accepted commands commit timing after creating their original snapshots.
            delivery.message.event =
                ServerEvent::GameSnapshot(session.game_snapshot(snapshot.you()));
        }
    }
    deliveries
}
