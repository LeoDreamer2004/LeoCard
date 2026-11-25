use super::configuration::{ConfigurationScroll, clamp_configuration_scroll};
use super::{
    AmbientBackgroundPlugin, ClickFeedbackPlugin, ConfirmationPlugin, GameHeaderPlugin,
    LiquidUiPlugin, ModalAnimationSet, ModalAnimations, PageTransitionPlugin, animate_cozy_modals,
    scroll_modal_content, update_cozy_checkbox_hover,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;
use leocard_ui::{animate_switches, update_help_tooltips};

pub(crate) struct ShellUiPlugin;
impl Plugin for ShellUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AmbientBackgroundPlugin,
            LiquidUiPlugin,
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
            (update_cozy_checkbox_hover, animate_switches)
                .after(animate_cozy_modals)
                .in_set(ClientUpdateSet::Animate),
        )
        .add_systems(
            Update,
            (
                clamp_configuration_scroll,
                scroll_modal_content::<ConfigurationScroll>,
                update_help_tooltips,
            )
                .chain()
                .in_set(ClientUpdateSet::Input),
        );
    }
}
