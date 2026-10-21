use super::state::LauncherState;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[derive(Component)]
pub(super) enum LauncherScroll {
    Versions,
    VersionDetail,
    Resources,
}

pub(super) fn scroll_launcher_lists(
    mut wheels: MessageReader<MouseWheel>,
    state: Res<LauncherState>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<LauncherScroll>,
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 40.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if !state.open || delta == 0.0 {
        return;
    }
    for (cursor, mut position, node) in &mut scrolls {
        if cursor.cursor_over() {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            position.y = (position.y - delta).clamp(0.0, maximum);
        }
    }
}

pub(super) fn remember_launcher_scroll(
    scrolls: Query<(&LauncherScroll, &ScrollPosition)>,
    mut state: ResMut<LauncherState>,
) {
    for (kind, position) in &scrolls {
        match kind {
            LauncherScroll::Versions => state.version_scroll_y = position.y,
            LauncherScroll::VersionDetail => state.detail_scroll_y = position.y,
            LauncherScroll::Resources => state.resource_scroll_y = position.y,
        }
    }
}
