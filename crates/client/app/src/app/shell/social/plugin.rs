use super::{
    AutoEggThrower, OpponentBadge, PlayerInteractionCooldown, animate_egg_barrages,
    animate_player_interactions, begin_shoe_hold, click_shoe_button,
    close_interaction_menu_on_outside_click, dispatch_social_actions, sync_egg_thrower_counts,
    sync_interaction_cooldown_masks, sync_opponent_badge_popups, sync_player_interactions,
    sync_spectator_view_menu, tick_auto_egg_thrower, tick_player_interaction_cooldown,
    update_shoe_holds,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use bevy::prelude::*;
use leocard_ui::SecondaryPressTarget;

pub(crate) struct SocialPlugin;

impl Plugin for SocialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInteractionCooldown::default())
            .init_resource::<AutoEggThrower>()
            .add_observer(begin_shoe_hold)
            .add_observer(click_shoe_button)
            .register_required_components_with::<OpponentBadge, SecondaryPressTarget>(|| {
                SecondaryPressTarget
            })
            .add_systems(Update, dispatch_social_actions.in_set(UiActionSet))
            .add_systems(
                Update,
                (
                    tick_player_interaction_cooldown,
                    update_shoe_holds,
                    (tick_auto_egg_thrower, sync_egg_thrower_counts).chain(),
                    close_interaction_menu_on_outside_click,
                    sync_interaction_cooldown_masks,
                    (sync_player_interactions, animate_egg_barrages).chain(),
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
