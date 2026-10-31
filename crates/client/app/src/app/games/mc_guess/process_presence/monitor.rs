use super::detection::mc_running;
use crate::app::runtime::ClientResource;
use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task, futures_lite::future};
use leocard_client::NetworkState;
use leocard_game_common::PlayerId;
use leocard_protocol::{ClientCommand, GameSnapshot, MatchId, McGuessCommand, RoomId};

const SCAN_INTERVAL: f32 = 5.0;

#[derive(Resource, Default)]
pub(super) struct McProcessMonitor {
    session: Option<(RoomId, MatchId, PlayerId)>,
    task: Option<Task<Option<bool>>>,
    elapsed: f32,
    reported: Option<bool>,
}

pub(super) fn monitor_mc_process(
    time: Res<Time>,
    mut client: Option<ResMut<ClientResource>>,
    mut monitor: ResMut<McProcessMonitor>,
) {
    let session = client.as_deref().and_then(|client| {
        if !matches!(client.0.state(), NetworkState::Connected(_)) {
            return None;
        }
        client
            .0
            .model()
            .game_snapshot()
            .and_then(GameSnapshot::mc_guess)
            .map(|game| (client.0.model().room_id(), game.match_id, game.you))
    });
    if monitor.session != session {
        *monitor = McProcessMonitor {
            session,
            elapsed: SCAN_INTERVAL,
            ..default()
        };
    }
    if session.is_none() {
        return;
    }
    if let Some(result) = monitor
        .task
        .as_mut()
        .and_then(|task| future::block_on(future::poll_once(task)))
    {
        monitor.task = None;
        if let Some(running) = result
            && monitor.reported != Some(running)
            && client
                .as_deref_mut()
                .expect("active session")
                .0
                .send(ClientCommand::Game(
                    McGuessCommand::ReportMcProcess { running }.into(),
                ))
        {
            monitor.reported = Some(running);
        }
    }
    monitor.elapsed += time.delta_secs();
    if monitor.task.is_none() && monitor.elapsed >= SCAN_INTERVAL {
        monitor.elapsed = 0.0;
        monitor.task = Some(IoTaskPool::get().spawn(async { mc_running() }));
    }
}
