use super::commands::is_decision;
use super::windows::pending_actions;
use crate::lifecycle::{HostedGameLifecycle, dispatch_client_command_without_timing};
use crate::{ConnectionId, Delivery};
use leocard_protocol::{ClientCommand, ClientMessage, GameSnapshot, ServerEvent, ShengjiPhaseView};
use std::time::Duration;

fn refresh<S: HostedGameLifecycle>(session: &mut S) {
    if session.room().closed || !session.game_started() {
        session.room_mut().action_timing.clear();
        return;
    }
    let Some(recipient) = session.room().players.first().map(|player| player.id) else {
        session.room_mut().action_timing.clear();
        return;
    };
    let snapshot = session.game_snapshot(recipient);
    let match_id = snapshot.match_id();
    let mut windows = pending_actions(&snapshot);
    // Bid/pass eligibility is private; merge individual recipient views only for
    // those phases. Public decision windows need just one snapshot.
    if matches!(&snapshot, GameSnapshot::Shengji(game)
        if matches!(game.phase, ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. }))
    {
        for player in &session.room().players {
            if player.id != recipient {
                windows.extend(pending_actions(&session.game_snapshot(player.id)));
            }
        }
    }
    // Removing unattended windows also hides their labels and guarantees that
    // reconnecting or taking over from auto-play starts a fresh human decision.
    windows.retain(|window| {
        session.room().players.iter().any(|player| {
            player.id == window.player
                && player.connected
                && !player.left
                && !player.is_bot
                && !player.auto_play
        })
    });
    session
        .room_mut()
        .action_timing
        .sync(S::KIND, match_id, windows);
}

fn decorate<S: HostedGameLifecycle>(session: &S, mut deliveries: Vec<Delivery>) -> Vec<Delivery> {
    let timing = session.room().action_timing.snapshot();
    for delivery in &mut deliveries {
        delivery.message.action_timing.clone_from(&timing);
        if session.game_started()
            && let ServerEvent::GameSnapshot(snapshot) = &delivery.message.event
        {
            // The command created its snapshot before its timing statistic was
            // committed. Re-project it, rather than adding client-side deltas.
            delivery.message.event =
                ServerEvent::GameSnapshot(session.game_snapshot(snapshot.you()));
        }
    }
    deliveries
}

pub(crate) fn dispatch_client_command<S: HostedGameLifecycle>(
    session: &mut S,
    connection: ConnectionId,
    message: ClientMessage,
) -> Vec<Delivery> {
    let player = match &message.command {
        ClientCommand::Game(command) if is_decision(command) => session
            .room()
            .players
            .iter()
            .find(|player| {
                player.connection == connection
                    && player.connected
                    && !player.left
                    && !player.is_bot
                    && !player.auto_play
            })
            .map(|player| player.id),
        _ => None,
    };
    let revision = session.room().revision;
    let joining = matches!(&message.command, ClientCommand::Join(_));
    let request = message.request_id;
    let deliveries = dispatch_client_command_without_timing(session, connection, message);
    let rejected = deliveries.iter().any(|delivery| {
        delivery.recipient == connection
            && delivery.message.in_reply_to == Some(request)
            && matches!(delivery.message.event, ServerEvent::Rejected { .. })
    });
    if session.room().revision != revision
        && !rejected
        && let Some(player) = player
        && let Some(millis) = session.room_mut().action_timing.complete(player)
    {
        let participant = session
            .room_mut()
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .expect("joined decision player");
        participant
            .game_profiles
            .action_times
            .record(S::KIND, millis);
    }
    if joining
        && !rejected
        && session.room().revision != revision
        && let Some(player) = session.room().player_id(connection)
    {
        // A reconnect can arrive before the next host tick has removed its old
        // window. Discard that partial decision even in this no-tick case.
        session.room_mut().action_timing.complete(player);
    }
    refresh(session);
    decorate(session, deliveries)
}

pub(crate) fn advance_with_timing<S: HostedGameLifecycle>(
    session: &mut S,
    elapsed: Duration,
    advance: fn(&mut S, Duration) -> Vec<Delivery>,
) -> Vec<Delivery> {
    session.room_mut().action_timing.advance(elapsed);
    let deliveries = advance(session, elapsed);
    // Automatic operations may change a window but are never human samples.
    refresh(session);
    decorate(session, deliveries)
}
