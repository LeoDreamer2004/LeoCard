use super::state::{GraphGuideScroll, GraphGuideState, advance_guide, sync_guide};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, animate_cozy_modals, scroll_modal_content};
use bevy::prelude::*;

pub(in super::super::super) struct GraphGuidePlugin;

impl Plugin for GraphGuidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphGuideState>()
            .add_systems(Update, advance_guide.in_set(ModalAnimationSet::Progress))
            .add_systems(
                Update,
                (sync_guide, scroll_modal_content::<GraphGuideScroll>)
                    .chain()
                    .after(advance_guide)
                    .before(animate_cozy_modals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
