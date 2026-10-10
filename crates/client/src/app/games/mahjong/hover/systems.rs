use super::super::MahjongTileMaterial;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use leocard_mahjong::MahjongTileKind;

#[derive(Component)]
pub(in super::super) struct MahjongHoverHandKind(pub MahjongTileKind);

#[derive(Component)]
pub(in super::super) struct MahjongMatchingTileKind(pub MahjongTileKind);

#[derive(Component)]
pub(in super::super) struct MahjongWaitPopupLink(pub Entity);

#[derive(Component)]
pub(in super::super) struct MahjongReadyHint;

pub(in super::super) struct MahjongReadyHintState {
    pub show_fans: bool,
    pub hovered: bool,
}

pub(in super::super) fn sync_mahjong_hover_hints(
    hands: Query<(&PickingInteraction, &MahjongHoverHandKind)>,
    popups: Query<(&PickingInteraction, &MahjongWaitPopupLink)>,
    matching_tiles: Query<(&MahjongMatchingTileKind, &MaterialNode<MahjongTileMaterial>)>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut visibility: Query<&mut Visibility>,
) {
    let hovered_kind = hands
        .iter()
        .find_map(|(interaction, kind)| hovered(interaction).then_some(kind.0));
    for (interaction, popup) in &popups {
        if let Ok(mut visible) = visibility.get_mut(popup.0) {
            let next = if hovered(interaction) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *visible != next {
                *visible = next;
            }
        }
    }
    for (kind, material_node) in &matching_tiles {
        let Some(material) = materials.get(&material_node.0) else {
            continue;
        };
        let strength = f32::from(material.params.z > 0.98 && hovered_kind == Some(kind.0));
        if material.lighting.z != strength
            && let Some(mut material) = materials.get_mut(&material_node.0)
        {
            material.lighting.z = strength;
        }
    }
}

fn hovered(interaction: &PickingInteraction) -> bool {
    matches!(
        interaction,
        PickingInteraction::Hovered | PickingInteraction::Pressed
    )
}
