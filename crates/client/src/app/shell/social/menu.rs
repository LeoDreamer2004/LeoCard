use super::*;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub(crate) fn tick_player_interaction_cooldown(
    time: Res<Time>,
    mut cooldown: ResMut<PlayerInteractionCooldown>,
) {
    if !cooldown.timers.is_empty() {
        cooldown.tick(time.delta_secs());
    }
}

pub(crate) fn close_interaction_menu_on_outside_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut ui: ResMut<UiState>,
    badges: Query<&Interaction, With<OpponentBadge>>,
    menus: Query<(&InteractionMenuPanel, &ComputedNode, &UiGlobalTransform)>,
) {
    let Some(open_player) = ui.social.interaction_menu_open else {
        return;
    };
    if !mouse.just_pressed(MouseButton::Left)
        || badges
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let Some(cursor) = windows
        .single()
        .ok()
        .and_then(Window::physical_cursor_position)
    else {
        return;
    };
    let inside = menus.iter().any(|(panel, node, transform)| {
        panel.0 == open_player && node.contains_point(*transform, cursor)
    });
    if !inside {
        ui.social.interaction_menu_open = None;
    }
}

pub(crate) fn sync_interaction_cooldown_masks(
    cooldown: Res<PlayerInteractionCooldown>,
    ui: Res<UiState>,
    assets: Res<UiAssets>,
    mut masks: Query<(&InteractionCooldownMask, &mut ImageNode, &mut Visibility)>,
) {
    for (mask, mut image, mut visibility) in &mut masks {
        let frame = if ui.social.interaction_menu_open == Some(mask.player) {
            (cooldown.fraction(mask.kind) * INTERACTION_COOLDOWN_MASK_FRAMES as f32).ceil() as usize
        } else {
            0
        };
        let expected_visibility = if frame == 0 {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
        if *visibility != expected_visibility {
            *visibility = expected_visibility;
        }
        if let Some(mask) = assets.social.interaction_cooldown_masks.get(frame)
            && image.image != *mask
        {
            image.image = mask.clone();
        }
    }
}

pub(crate) fn sync_opponent_badge_popups(
    ui: Res<UiState>,
    badges: Query<(&Interaction, &OpponentBadge)>,
    mut visibility: Query<&mut Visibility>,
) {
    for (interaction, badge) in &badges {
        let menu_open = ui.social.interaction_menu_open == Some(badge.player);
        if let Some(score_popup) = badge.score_popup
            && let Ok(mut popup) = visibility.get_mut(score_popup)
        {
            let expected = if !menu_open
                && matches!(interaction, Interaction::Hovered | Interaction::Pressed)
            {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *popup != expected {
                *popup = expected;
            }
        }
        if let Ok(mut menu) = visibility.get_mut(badge.interaction_menu) {
            let expected = if menu_open {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *menu != expected {
                *menu = expected;
            }
        }
    }
}
