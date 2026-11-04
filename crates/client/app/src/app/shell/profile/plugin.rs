use super::actions::dispatch_profile_actions;
use super::scroll::{scroll_archive, update_archive_edge};
use super::{ProfileMotion, advance_profile_modal, update_profile_tab_hover};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::UiActionSet;
use crate::app::shell::{ModalAnimationSet, animate_cozy_modals};
use bevy::prelude::*;

pub(crate) struct ProfilePlugin;

impl Plugin for ProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProfileMotion>()
            .add_systems(Update, dispatch_profile_actions.in_set(UiActionSet))
            .add_systems(Update, scroll_archive.in_set(ClientUpdateSet::Input))
            .add_systems(Update, update_archive_edge.in_set(ClientUpdateSet::Animate))
            .add_systems(
                Update,
                advance_profile_modal.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                update_profile_tab_hover
                    .after(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
