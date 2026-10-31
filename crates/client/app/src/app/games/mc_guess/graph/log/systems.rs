use super::super::super::McGuessAssets;
use super::state::{GraphLog, GraphLogMessage, GraphLogRoot, GraphLogScroll, LogMessage};
use super::view::render_log;
use crate::app::runtime::{AvatarImages, ClientResource, UiAssets};
use crate::app::shell::UiState;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_protocol::GameSnapshot;
use leocard_ui::{ScrollContentHeight, UiComposer};

pub(super) fn sync_messages(
    client: Option<Res<ClientResource>>,
    mut log: ResMut<GraphLog>,
    avatars: Res<AvatarImages>,
) {
    let Some((game, graph)) = client
        .as_deref()
        .and_then(|client| {
            client
                .0
                .model()
                .game_snapshot()
                .and_then(GameSnapshot::mc_guess)
        })
        .and_then(|game| game.graph.as_ref().map(|graph| (game, graph)))
    else {
        if log.key.is_some() {
            *log = GraphLog::default();
        }
        return;
    };
    let key = (game.match_id, game.progress.game_index);
    if log.key != Some(key) {
        *log = GraphLog {
            key: Some(key),
            ..default()
        };
    }
    let received = log.messages.len();
    for record in graph.guesses.iter().skip(received) {
        let player = game
            .players
            .iter()
            .find(|player| player.player.id == record.guess.player);
        let avatar = player.and_then(|player| {
            if player.player.id == game.you {
                avatars.local.clone()
            } else {
                player
                    .player
                    .avatar
                    .and_then(|id| avatars.remote.get(&id).cloned())
            }
        });
        log.messages.push(LogMessage {
            name: player
                .map(|player| player.player.name.clone())
                .unwrap_or_else(|| "离线玩家".to_owned()),
            avatar,
            node: record.node,
            guess: record.guess.result.name().to_owned(),
            correct: record.guess.result.is_correct(),
            combo: record.combo,
        });
    }
    if log.messages.len() != received {
        log.version += 1;
    }
}

pub(super) fn sync_view(
    mut commands: UiComposer,
    log: Res<GraphLog>,
    roots: Query<(Entity, &GraphLogRoot)>,
    assets: Res<UiAssets>,
    game_assets: Res<McGuessAssets>,
    ui: Res<UiState>,
) {
    let visible = !ui.shop.open && !ui.achievements.open && !ui.leaving_room;
    let mut retained = false;
    for (_, root) in &roots {
        if visible && Some(root.key) == log.key && root.version == log.version {
            retained = true;
        } else {
            commands.entity(root.layer).despawn();
        }
    }
    if retained {
        return;
    }
    if visible && let Some(key) = log.key {
        render_log(&mut commands, key, &log, &assets, &game_assets);
    }
}

pub(super) fn remember_scroll(
    mut log: ResMut<GraphLog>,
    scrolls: Query<(&ComputedNode, &ScrollPosition, &ScrollContentHeight), With<GraphLogScroll>>,
) {
    for (node, position, height) in &scrolls {
        if node.size().y == 0.0 {
            continue;
        }
        let maximum = height.maximum(node);
        log.follow_tail = maximum - position.y <= 4.0;
        log.scroll_y = position.y.clamp(0.0, maximum);
    }
}

pub(super) fn settle_scroll(
    mut log: ResMut<GraphLog>,
    messages: Query<&ComputedNode, With<GraphLogMessage>>,
    mut scrolls: Query<
        (&ComputedNode, &mut ScrollPosition, &mut ScrollContentHeight),
        With<GraphLogScroll>,
    >,
) {
    let content_height = messages
        .iter()
        .map(|node| node.size().y * node.inverse_scale_factor())
        .sum();
    for (node, mut position, mut height) in &mut scrolls {
        height.0 = content_height;
        let maximum = height.maximum(node);
        position.y = if log.follow_tail {
            maximum
        } else {
            position.y.clamp(0.0, maximum)
        };
        log.scroll_y = position.y;
    }
}

pub(super) fn animate_panel(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut log: ResMut<GraphLog>,
    mut panels: Query<(&RelativeCursorPosition, &mut Node), With<GraphLogRoot>>,
) {
    for (cursor, mut node) in &mut panels {
        if log.expanded
            && (mouse.just_pressed(MouseButton::Left) || touches.any_just_pressed())
            && !cursor.cursor_over()
        {
            log.toggle();
        }
        let target = if log.expanded { 320.0 } else { 96.0 };
        log.height += (target - log.height) * (1.0 - (-18.0 * time.delta_secs()).exp());
        node.height = px(log.height);
    }
}
