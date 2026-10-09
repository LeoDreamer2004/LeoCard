use super::{
    AmbientBackgroundPlugin, ClickFeedbackPlugin, ConfirmationPlugin, ModalAnimationSet,
    ModalAnimations, PageTransitionPlugin, animate_cozy_modals,
};
use crate::app::runtime::ClientUpdateSet;
use bevy::prelude::*;

pub(crate) struct ShellUiPlugin;
impl Plugin for ShellUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AmbientBackgroundPlugin,
            ClickFeedbackPlugin,
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
        );
    }
}
