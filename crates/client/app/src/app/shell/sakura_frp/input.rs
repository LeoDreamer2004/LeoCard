use super::state::SakuraState;
use crate::app::runtime::PageErrorState;
use crate::app::shell::UiState;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::RelativeCursorPosition;
use leocard_ui::{TextInputAction, TextInputEvent};

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum SakuraScroll {
    Tunnels,
    Nodes,
}

pub(super) fn scroll_sakura_lists(
    mut wheels: MessageReader<MouseWheel>,
    state: Res<SakuraState>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        With<SakuraScroll>,
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

pub(super) fn remember_sakura_scroll(
    scrolls: Query<(&SakuraScroll, &ScrollPosition)>,
    mut state: ResMut<SakuraState>,
) {
    for (kind, position) in &scrolls {
        match kind {
            SakuraScroll::Tunnels => state.tunnel_scroll_y = position.y,
            SakuraScroll::Nodes => state.node_scroll_y = position.y,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum SakuraInput {
    Token,
    Name,
    LocalIp,
    LocalPort,
    Remote,
}

pub(super) fn sync_sakura_inputs(
    inputs: Query<(&SakuraInput, &EditableText)>,
    mut state: ResMut<SakuraState>,
    mut ui: ResMut<UiState>,
) {
    for (kind, input) in &inputs {
        let value = match kind {
            SakuraInput::Token => &mut state.token,
            SakuraInput::Name => &mut state.name,
            SakuraInput::LocalIp => &mut state.local_ip,
            SakuraInput::LocalPort => &mut state.local_port,
            SakuraInput::Remote => &mut state.remote,
        };
        if input.value() != value.as_str() {
            *value = input.value().to_string();
            ui.dirty = true;
        }
    }
}

pub(super) fn submit_sakura_login(
    mut events: MessageReader<TextInputEvent>,
    inputs: Query<(&SakuraInput, &EditableText)>,
    mut state: ResMut<SakuraState>,
    mut error: ResMut<PageErrorState>,
    mut ui: ResMut<UiState>,
) {
    for event in events.read() {
        if state.open
            && state.client.is_none()
            && !state.busy
            && event.action == TextInputAction::Submit
            && let Ok((SakuraInput::Token, input)) = inputs.get(event.entity)
        {
            state.token = input.value().to_string();
            error.error = None;
            super::account::login(&mut state);
            ui.dirty = true;
        }
    }
}
