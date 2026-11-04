use super::super::super::McGuessAssets;
use super::state::{GraphLog, GraphLogMessage, GraphLogRoot, GraphLogScroll};
use super::view::render_log;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiState;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::{ScrollContentHeight, UiComposer};

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
