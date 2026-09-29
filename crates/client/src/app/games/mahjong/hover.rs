//! 手牌悬停时的同牌提示与弃牌听牌预览。

use super::{
    MahjongAssets, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    add_mahjong_tile_material,
};
use crate::app::presentation::{MUTED, TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::{
    MahjongKongKind, MahjongMeldKind, MahjongPlayerId, MahjongTileKind, Meld, is_complete_hand,
};
use leocard_protocol::{MahjongPublicMeldView, MahjongSnapshot};
use std::collections::HashMap;

#[derive(Component)]
pub(super) struct MahjongHoverHandKind(pub MahjongTileKind);

#[derive(Component)]
pub(super) struct MahjongMatchingTileKind(pub MahjongTileKind);

#[derive(Component)]
pub(super) struct MahjongWaitPopupLink(pub Entity);

pub(super) fn add_mahjong_wait_popup(
    commands: &mut Commands,
    hand_tile: Entity,
    waits: &[(MahjongTileKind, u8)],
    game_assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
    assets: &UiAssets,
) {
    if waits.is_empty() {
        return;
    }
    let columns = waits.len().min(10);
    let width = columns as f32 * 31.0 + 14.0;
    let popup = spawn_node(
        commands,
        hand_tile,
        Node {
            position_type: PositionType::Absolute,
            left: px((50.0 - width) * 0.5),
            bottom: px(76),
            width: px(width),
            padding: UiRect::all(px(7)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(5),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(Color::srgb(0.07, 0.18, 0.23).with_alpha(0.96)),
    );
    commands.entity(popup).insert((
        Visibility::Hidden,
        GlobalZIndex(1600),
        FocusPolicy::Pass,
        BoxShadow::new(Color::BLACK.with_alpha(0.4), px(1), px(5), px(0), px(9)),
    ));
    commands
        .entity(hand_tile)
        .insert(MahjongWaitPopupLink(popup));
    for (index, &(kind, remaining)) in waits.iter().enumerate() {
        let cell = spawn_node(
            commands,
            popup,
            Node {
                width: px(31),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(2),
                ..default()
            },
            None,
        );
        commands.entity(cell).insert(FocusPolicy::Pass);
        add_mahjong_tile_material(
            commands,
            cell,
            MahjongTileVisual {
                kind: Some(kind),
                size: MahjongTileSize::Mini,
                index,
                highlighted: false,
                deal: None,
                relative: 0,
            },
            game_assets,
            materials,
        );
        let count = add_text(
            commands,
            cell,
            remaining.to_string(),
            13.0,
            if remaining == 0 { MUTED } else { TEXT },
            assets,
        );
        commands.entity(count).insert(FocusPolicy::Pass);
    }
}

pub(super) fn sync_mahjong_hover_hints(
    hands: Query<(
        &Interaction,
        &MahjongHoverHandKind,
        Option<&MahjongWaitPopupLink>,
    )>,
    matching_tiles: Query<(&MahjongMatchingTileKind, &MaterialNode<MahjongTileMaterial>)>,
    mut materials: ResMut<Assets<MahjongTileMaterial>>,
    mut visibility: Query<&mut Visibility>,
) {
    let mut hovered_kind = None;
    for (interaction, kind, popup) in &hands {
        let hovered = matches!(interaction, Interaction::Hovered | Interaction::Pressed);
        if hovered {
            hovered_kind = Some(kind.0);
        }
        if let Some(popup) = popup
            && let Ok(mut visible) = visibility.get_mut(popup.0)
        {
            let next = if hovered {
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

pub(super) fn mahjong_discard_waits(
    game: &MahjongSnapshot,
) -> HashMap<MahjongTileKind, Vec<(MahjongTileKind, u8)>> {
    let visible = visible_kind_counts(game);
    let Some(own) = game.players.iter().find(|player| player.id == game.you) else {
        return HashMap::new();
    };
    let Some(melds) = own.melds.iter().map(core_meld).collect::<Option<Vec<_>>>() else {
        return HashMap::new();
    };
    let mut waits = HashMap::new();
    for (discard_index, discard) in game.your_hand.iter().enumerate() {
        let discard_kind = discard.kind();
        if waits.contains_key(&discard_kind) {
            continue;
        }
        let mut concealed = game
            .your_hand
            .iter()
            .enumerate()
            .filter_map(|(index, tile)| (index != discard_index).then_some(tile.kind()))
            .collect::<Vec<_>>();
        let mut kinds = Vec::new();
        for (index, &visible_count) in visible.iter().enumerate() {
            let kind = MahjongTileKind::from_index34(index).expect("34 种非花牌");
            concealed.push(kind);
            if is_complete_hand(&concealed, &melds) {
                kinds.push((kind, 4_u8.saturating_sub(visible_count)));
            }
            concealed.pop();
        }
        waits.insert(discard_kind, kinds);
    }
    waits
}

fn core_meld(meld: &MahjongPublicMeldView) -> Option<Meld> {
    let tile = meld.tile?;
    let source = MahjongPlayerId(meld.claimed_from.map_or(0, |player| player.0 as usize));
    Some(match meld.kind {
        MahjongMeldKind::Chow => {
            let MahjongTileKind::Suited { suit, rank } = tile else {
                return None;
            };
            Meld::chow(suit, rank, source)
        }
        MahjongMeldKind::Pung => Meld::pung(tile, source),
        MahjongMeldKind::Kong(MahjongKongKind::Melded) => Meld::melded_kong(tile, source),
        MahjongMeldKind::Kong(MahjongKongKind::Concealed) => Meld::concealed_kong(tile),
    })
}

fn visible_kind_counts(game: &MahjongSnapshot) -> [u8; 34] {
    let mut counts = [0_u8; 34];
    let mut add = |kind: MahjongTileKind| {
        if let Some(index) = kind.index34() {
            counts[index] = counts[index].saturating_add(1);
        }
    };
    for tile in &game.your_hand {
        add(tile.kind());
    }
    for discard in game
        .discards
        .iter()
        .filter(|discard| discard.claimed_by.is_none())
    {
        add(discard.tile.kind());
    }
    for player in &game.players {
        if player.id != game.you
            && let Some(revealed) = &player.revealed_hand
        {
            for tile in revealed {
                add(tile.kind());
            }
        }
        for meld in &player.melds {
            if player.id != game.you
                && matches!(meld.kind, MahjongMeldKind::Kong(MahjongKongKind::Concealed))
            {
                continue;
            }
            if let Some(core) = core_meld(meld) {
                for kind in core.tile_kinds() {
                    add(kind);
                }
            }
        }
    }
    counts
}
