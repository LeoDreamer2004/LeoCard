use super::{LiquidMaterial, animate_liquid, capture_ripples, initialize_liquid};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{ModalAnimationSet, PageTransitionSet, UiActionSet};
use bevy::prelude::*;
use bevy::ui_render::UiMaterialPlugin;

pub(crate) struct LiquidUiPlugin;

impl Plugin for LiquidUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<LiquidMaterial>::default())
            .add_systems(Update, capture_ripples.before(UiActionSet))
            .add_systems(
                Update,
                (initialize_liquid, animate_liquid)
                    .chain()
                    .after(PageTransitionSet)
                    .after(ModalAnimationSet::Visuals)
                    .in_set(ClientUpdateSet::Animate),
            );
    }
}
