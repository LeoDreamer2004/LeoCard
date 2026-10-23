#[cfg(not(target_os = "android"))]
use super::switch::{CozySwitchThumb, animate_cozy_switches};
use super::{
    AmbientBackgroundPlugin, ClickFeedbackPlugin, ConfirmationPlugin, GameHeaderPlugin,
    ModalAnimationSet, ModalAnimations, PageTransitionPlugin, animate_cozy_modals,
    update_cozy_checkbox_hover,
};
#[cfg(not(target_os = "android"))]
use crate::app::presentation::UiSceneProperties;
use crate::app::presentation::update_rule_help_tooltips;
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct ShellUiPlugin;
impl Plugin for ShellUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AmbientBackgroundPlugin,
            ClickFeedbackPlugin,
            GameHeaderPlugin,
            PageTransitionPlugin,
            ConfirmationPlugin,
        ))
        .init_resource::<ModalAnimations>()
        .configure_sets(
            Update,
            (ModalAnimationSet::Progress, ModalAnimationSet::Visuals)
                .chain()
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            animate_cozy_modals.in_set(ModalAnimationSet::Visuals),
        )
        .add_systems(
            Update,
            update_cozy_checkbox_hover
                .after(animate_cozy_modals)
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            update_rule_help_tooltips.in_set(ClientUpdateSet::Input),
        );
        #[cfg(not(target_os = "android"))]
        {
            app.world_mut()
                .resource_mut::<UiSceneProperties>()
                .preserve_with::<CozySwitchThumb>(CozySwitchThumb::same_owner);
            app.add_systems(
                Update,
                animate_cozy_switches.in_set(ClientUpdateSet::Animate),
            );
        }
    }
}
