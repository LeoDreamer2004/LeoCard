#[cfg(target_os = "android")]
use super::input::handle_header_taps;
#[cfg(not(target_os = "android"))]
use super::state::GameHeaderAvatar;
use super::state::{GameHeaderItem, GameHeaderMotion};
use crate::app::runtime::ClientUpdateSet;
#[cfg(target_os = "android")]
use crate::app::shell::UiActionSet;
#[cfg(not(target_os = "android"))]
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
#[cfg(not(target_os = "android"))]
use bevy::window::PrimaryWindow;
use leocard_ui::{TransitionVisuals, UiComposer, fade_panel, update_button_highlights};

pub(in crate::app::shell::ui) struct GameHeaderPlugin;

impl Plugin for GameHeaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (advance_header, animate_items)
                .chain()
                .after(update_button_highlights)
                .in_set(ClientUpdateSet::Animate),
        );
        #[cfg(not(target_os = "android"))]
        app.add_systems(
            Update,
            track_header_hover
                .before(advance_header)
                .in_set(ClientUpdateSet::Animate),
        );
        #[cfg(target_os = "android")]
        app.add_systems(
            Update,
            handle_header_taps
                .before(UiActionSet)
                .in_set(ClientUpdateSet::Input),
        );
    }
}

#[cfg(not(target_os = "android"))]
fn track_header_hover(
    windows: Query<&Window, With<PrimaryWindow>>,
    avatars: Query<&PickingInteraction, With<GameHeaderAvatar>>,
    mut headers: Query<(&mut GameHeaderMotion, &ComputedNode, &UiGlobalTransform)>,
) {
    let cursor = windows
        .single()
        .ok()
        .filter(|window| window.focused)
        .and_then(Window::physical_cursor_position);
    let hovered = cursor.is_some()
        && avatars
            .iter()
            .any(|state| *state != PickingInteraction::None);
    for (mut motion, node, transform) in &mut headers {
        let inside = cursor.is_some_and(|point| node.contains_point(*transform, point));
        let expanded = hovered || motion.expanded && inside;
        if motion.expanded != expanded {
            motion.expanded = expanded;
        }
    }
}

fn advance_header(time: Res<Time>, mut headers: Query<&mut GameHeaderMotion>) {
    for mut motion in &mut headers {
        let direction = if motion.expanded { 1.0 } else { -1.0 };
        let progress = (motion.progress + direction * time.delta_secs().min(1.0 / 30.0) / 0.28)
            .clamp(0.0, 1.0);
        if motion.progress != progress {
            motion.progress = progress;
        }
    }
}

fn animate_items(
    mut commands: UiComposer,
    headers: Query<&GameHeaderMotion>,
    mut items: Query<(
        Entity,
        &ChildOf,
        &mut GameHeaderItem,
        &mut Node,
        &mut UiTransform,
    )>,
    children: Query<&Children>,
    mut visuals: TransitionVisuals,
) {
    for (entity, parent, mut item, mut node, mut transform) in &mut items {
        let Ok(motion) = headers.get(parent.parent()) else {
            continue;
        };
        let progress = (motion.progress * 1.24 - item.slot as f32 * 0.12).clamp(0.0, 1.0);
        if item.applied == Some(progress) {
            continue;
        }
        item.applied = Some(progress);
        let visible = progress * progress * (3.0 - 2.0 * progress);
        let display = if progress == 0.0 {
            Display::None
        } else {
            Display::Flex
        };
        if node.display != display {
            node.display = display;
        }
        let mut expected =
            UiTransform::from_translation(Val2::px(item.folded_offset() * (1.0 - visible), 0.0));
        expected.scale = Vec2::splat(0.6 + visible * 0.4);
        if *transform != expected {
            *transform = expected;
        }
        fade_panel(entity, visible, &children, &mut visuals, &mut commands);
    }
}
