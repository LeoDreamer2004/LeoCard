use super::super::guide::GraphGuideState;
use super::state::{GraphLog, GraphLogScroll};
use super::systems::{animate_panel, remember_scroll, settle_scroll, sync_messages, sync_view};
use crate::app::runtime::ClientUpdateSet;
use crate::app::shell::{UiState, scroll_modal_content};
use bevy::prelude::*;
use bevy::ui::UiSystems;
use leocard_ui::{drag_scrollbar, update_scrollbar};

pub(in super::super::super) struct GraphLogPlugin;

impl Plugin for GraphLogPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphLog>()
            .add_systems(Update, sync_messages.in_set(ClientUpdateSet::Sync))
            .add_systems(
                Update,
                (animate_panel, sync_view)
                    .chain()
                    .in_set(ClientUpdateSet::Animate),
            )
            .add_systems(
                Update,
                (
                    scroll_modal_content::<GraphLogScroll>.run_if(
                        |guide: Res<GraphGuideState>, ui: Res<UiState>| {
                            !guide.visible() && !ui.settings.open && !ui.profile.open
                        },
                    ),
                    remember_scroll,
                )
                    .chain()
                    .after(drag_scrollbar)
                    .in_set(ClientUpdateSet::Input),
            )
            .add_systems(
                PostUpdate,
                settle_scroll
                    .after(UiSystems::Layout)
                    .before(update_scrollbar),
            );
    }
}
