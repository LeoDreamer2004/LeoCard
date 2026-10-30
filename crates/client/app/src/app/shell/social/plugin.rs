use super::{
    OpponentBadge, PlayerInteractionCooldown, animate_player_interactions,
    close_interaction_menu_on_outside_click, dispatch_social_actions,
    sync_interaction_cooldown_masks, sync_opponent_badge_popups, sync_player_interactions,
    sync_spectator_view_menu, tick_player_interaction_cooldown,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::SecondaryPressTarget;

pub(crate) struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInteractionCooldown::default())
            .register_required_components_with::<OpponentBadge, SecondaryPressTarget>(|| {
                SecondaryPressTarget
            })
            .add_systems(Update, dispatch_social_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (
                    tick_player_interaction_cooldown,
                    close_interaction_menu_on_outside_click,
                    sync_interaction_cooldown_masks,
                    sync_player_interactions,
                    sync_spectator_view_menu,
                )
                    .in_set(ClientUpdateSet::Sync),
            )
            .add_systems(
                Update,
                (sync_opponent_badge_popups, animate_player_interactions)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
