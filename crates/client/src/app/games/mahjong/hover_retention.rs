//! 重建牌桌前保留麻将手牌的悬停演出。

use super::{MahjongHandTile, MahjongReadyHint, MahjongUiState};
use crate::app::runtime::ClientResource;
use bevy::ecs::system::SystemParam;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::GameSnapshot;

#[derive(SystemParam)]
pub(crate) struct MahjongHoverRetention<'w, 's> {
    ready_hints: Query<'w, 's, &'static PickingInteraction, With<MahjongReadyHint>>,
    tiles: Query<
        'w,
        's,
        (
            &'static MahjongHandTile,
            Option<&'static PickingInteraction>,
        ),
    >,
}

impl MahjongHoverRetention<'_, '_> {
    pub(crate) fn retain(&self, client: Option<&ClientResource>, ui: &mut MahjongUiState) {
        ui.hand_hover_lifts.clear();
        ui.ready_hint_hovered = false;
        let Some(ClientPhaseRef::Playing(GameSnapshot::Mahjong(game))) =
            client.map(|client| client.0.model().phase())
        else {
            return;
        };
        if ui.observed_table.key != Some((game.match_id, game.sequence_index)) {
            return;
        }
        ui.ready_hint_hovered = self.ready_hints.iter().any(|interaction| {
            matches!(
                interaction,
                PickingInteraction::Hovered | PickingInteraction::Pressed
            )
        });
        if ui.observed_table.state.hand != game.your_hand {
            return;
        }
        ui.hand_hover_lifts
            .extend(self.tiles.iter().filter(|(tile, _)| tile.lift > 0.0).map(
                |(tile, interaction)| {
                    (
                        tile.index,
                        (tile.lift, interaction.copied().unwrap_or_default()),
                    )
                },
            ));
    }
}
