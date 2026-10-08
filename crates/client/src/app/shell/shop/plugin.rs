use super::{
    actions::dispatch_shop_actions,
    page::{ItemRemaining, ShopList, remaining_label},
    sync::{publish_coins, sync_coins},
};
use crate::app::shell::{UiActionSet, UiState, scroll_modal_content};
use bevy::prelude::*;
use leocard_client::PlayerEconomy;

pub(crate) struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, sync_coins);
        app.add_systems(
            Update,
            (
                dispatch_shop_actions.in_set(UiActionSet),
                update_remaining,
                publish_coins,
                scroll_modal_content::<ShopList>,
            ),
        );
    }
}

fn update_remaining(
    economy: Res<PlayerEconomy>,
    mut ui: ResMut<UiState>,
    mut labels: Query<(&mut ItemRemaining, &mut Text)>,
) {
    for (mut marker, mut text) in &mut labels {
        let active = economy.active(marker.item);
        if active != marker.active {
            marker.active = active;
            ui.dirty = true;
        }
        let label = remaining_label(&economy, marker.item);
        if text.0 != label {
            text.0 = label;
        }
    }
}
