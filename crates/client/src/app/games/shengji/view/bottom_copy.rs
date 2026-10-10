use super::super::ShengjiPresentationState;
use super::super::presentation::shengji_player_route_anchor;
use super::{ShengjiCardSize, add_shengji_card_row, shengji_display_trump};
use crate::app::presentation::{ACCENT, MUTED, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use bevy::prelude::*;
use leocard_protocol::{PlayerId, ShengjiPhaseView, ShengjiSnapshot};

pub(super) fn add_shengji_bottom_copy_decision(
    commands: &mut Commands,
    area: Entity,
    game: &ShengjiSnapshot,
    player: PlayerId,
    assets: &UiAssets,
) -> bool {
    if !matches!(game.phase, ShengjiPhaseView::BottomCopying { .. }) {
        return false;
    }
    let Some(_) = game
        .bottom_copy_decisions
        .iter()
        .find(|decision| decision.player == player && decision.cards.is_none())
    else {
        return false;
    };
    let holder = spawn_node(
        commands,
        area,
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    add_text(commands, holder, "不抄底", 36.0, MUTED, assets);
    true
}

pub(super) fn add_shengji_bottom_copy_reveal(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    presentation: &ShengjiPresentationState,
    assets: &UiAssets,
) {
    let Some((player, cards)) = presentation.bottom_copy_reveal() else {
        return;
    };
    if !matches!(game.phase, ShengjiPhaseView::BottomCopyBurying { .. }) {
        return;
    }
    let anchor = shengji_player_route_anchor(game, player).expect("抄底玩家属于本局");
    let width = 70.0 + cards.len().saturating_sub(1) as f32 * 27.0;
    let holder = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(anchor.x),
            top: percent(anchor.y),
            width: px(width),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    commands.entity(holder).insert((
        UiTransform::from_translation(Val2::px(-width * 0.5, -60.0)),
        GlobalZIndex(530),
    ));
    add_text(commands, holder, "抄底", 17.0, ACCENT, assets);
    add_shengji_card_row(
        commands,
        holder,
        cards,
        ShengjiCardSize::Seat,
        shengji_display_trump(game),
        assets,
    );
}
