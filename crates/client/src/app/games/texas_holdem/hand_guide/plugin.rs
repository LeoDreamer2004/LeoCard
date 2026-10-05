use super::state::{
    TexasHandGuideScroll, TexasHandGuideState, advance_texas_hand_guide, sync_texas_hand_guide,
};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, animate_cozy_modals, scroll_modal_content};
use bevy::prelude::*;

pub(in super::super) struct TexasHandGuidePlugin;

impl Plugin for TexasHandGuidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TexasHandGuideState>()
            .add_systems(
                Update,
                advance_texas_hand_guide.in_set(ModalAnimationSet::Progress),
            )
            .add_systems(
                Update,
                (
                    sync_texas_hand_guide,
                    scroll_modal_content::<TexasHandGuideScroll>,
                )
                    .chain()
                    .after(advance_texas_hand_guide)
                    .before(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
