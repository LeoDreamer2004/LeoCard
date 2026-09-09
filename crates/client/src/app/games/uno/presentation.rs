//! UNO 出牌、摸牌、换牌、翻面、调色与转向演出。

mod animation;
mod flight;
mod flip;
mod palette;
mod reverse;

use super::{
    UNO_ACTION_AREA_BOTTOM, UNO_ACTION_AREA_HEIGHT, UNO_DISCARD_OFFSETS, UNO_FLYING_CARD_HEIGHT,
    UNO_FLYING_CARD_WIDTH, UNO_PALETTE_EFFECT_DURATION, UNO_REVERSE_EFFECT_DURATION, UnoAssets,
    UnoAudioState, UnoDiscardCard, UnoDiscardPileAnchor, UnoDrawPileAnchor, UnoFlipCard,
    UnoFlipOverlay, UnoFlipTarget, UnoFlyingCard, UnoPaletteColorRing, UnoPaletteEffect,
    UnoPaletteMaterial, UnoPaletteParticle, UnoPaletteSelectedSector, UnoPresentationState,
    UnoReverseArrow, uno_card_handle, uno_should_show_reverse_effect, uno_ui_color,
};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{PlayerAvatarAnchor, PlayerInteractionLayer};
pub(crate) use animation::*;
use bevy::prelude::*;
pub(crate) use flight::*;
use flip::*;
use leocard_protocol::{PlayerId, UnoEvent, UnoSnapshot};
use leocard_uno::{UnoCard, UnoDirection};
use palette::*;
use reverse::*;
use std::collections::HashMap;

/// UNO 最后一张牌的飞行动画结束后，完整公开牌桌两秒再进入结算。
pub(super) const UNO_PLAY_CARD_DURATION: f32 = 0.58;
pub(super) const UNO_FINISH_REVEAL_DURATION: f32 = UNO_PLAY_CARD_DURATION + 2.0;

struct UnoPresentationScene {
    layer: Entity,
    layer_size: Vec2,
    draw: Vec2,
    discard: Vec2,
    discard_top_pose: (f32, f32, f32),
    players: HashMap<PlayerId, Vec2>,
}

impl UnoPresentationScene {
    fn resolve(
        layer: Entity,
        layer_node: &ComputedNode,
        layer_transform: &UiGlobalTransform,
        draw: (&ComputedNode, &UiGlobalTransform),
        discard: (&ComputedNode, &UiGlobalTransform),
        game: &UnoSnapshot,
        anchors: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    ) -> Option<Self> {
        let draw = uno_anchor_in_layer(draw.0, draw.1, layer_node, layer_transform)?;
        let discard = uno_anchor_in_layer(discard.0, discard.1, layer_node, layer_transform)?;
        let players = game
            .players
            .iter()
            .map(|player| {
                uno_player_anchor_in_layer(player.id, layer_node, layer_transform, anchors)
                    .map(|position| (player.id, position))
            })
            .collect::<Option<HashMap<_, _>>>()?;
        Some(Self {
            layer,
            layer_size: layer_node.size() * layer_node.inverse_scale_factor(),
            draw,
            discard,
            discard_top_pose: uno_discard_pose(game.discard_top),
            players,
        })
    }

    fn player(&self, player: PlayerId) -> Vec2 {
        self.players.get(&player).copied().unwrap_or(self.discard)
    }

    fn discard_target(&self, card: UnoCard) -> Vec2 {
        let pose = uno_discard_pose(card);
        self.discard
            + Vec2::new(
                pose.0 - self.discard_top_pose.0,
                pose.1 - self.discard_top_pose.1,
            )
    }
}

pub(super) fn sync_uno_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut presentation: ResMut<UnoPresentationState>,
) {
    let Some(client) = client.as_deref_mut() else {
        presentation.events.clear();
        return;
    };
    presentation
        .events
        .extend(client.0.model_mut().take_uno_events());
    if client.0.model().uno_game().is_none() {
        presentation.events.clear();
    }
}
pub(crate) fn spawn_uno_presentation_effects(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    ui_assets: Res<UiAssets>,
    game_assets: Res<UnoAssets>,
    mut palette_materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut presentation: ResMut<UnoPresentationState>,
    mut audio: ResMut<UnoAudioState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    players: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    draws: Query<(&ComputedNode, &UiGlobalTransform), With<UnoDrawPileAnchor>>,
    discards: Query<(&ComputedNode, &UiGlobalTransform), With<UnoDiscardPileAnchor>>,
    flip_targets: Query<(Entity, &UnoFlipTarget, &ImageNode, &UiTransform)>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    else {
        presentation.events.clear();
        presentation.last_snapshot = None;
        return;
    };
    let previous_game = presentation
        .last_snapshot
        .as_ref()
        .filter(|previous| previous.match_id == game.match_id)
        .cloned();
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    if layer_node.size().min_element() <= 1.0 {
        return;
    }
    let Ok((draw_node, draw_transform)) = draws.single() else {
        return;
    };
    let Ok((discard_node, discard_transform)) = discards.single() else {
        return;
    };
    // `rebuild_ui` 会在收到权威快照时重建牌桌；同一帧中新节点尚未经过
    // Bevy 的布局阶段，ComputedNode/UiGlobalTransform 仍指向左上角。等到下一帧
    // 所有锚点拥有真实尺寸后再消费事件，否则整组动画会被画在 (0, 0)。
    let Some(scene) = UnoPresentationScene::resolve(
        layer,
        layer_node,
        layer_transform,
        (draw_node, draw_transform),
        (discard_node, discard_transform),
        game,
        &players,
    ) else {
        return;
    };

    while let Some(event) = presentation.events.pop_front() {
        audio.queue_event(&event, game.you);
        match event {
            UnoEvent::CardPlayed {
                player,
                card,
                chosen_color,
                play_index,
                play_count,
            } => {
                let source = scene.player(player);
                let card_pose = uno_discard_pose(card);
                let card_target = scene.discard_target(card);
                spawn_uno_flying_card(
                    &mut commands,
                    scene.layer,
                    uno_card_handle(&game_assets, card),
                    Some(card),
                    source,
                    card_target,
                    0.0,
                    false,
                    card.copy() as usize,
                    card_pose.2,
                );
                if uno_should_show_reverse_effect(card, play_index, play_count) {
                    spawn_uno_reverse_effect(
                        &mut commands,
                        scene.layer,
                        layer_node,
                        layer_transform,
                        game,
                        &players,
                        card.color()
                            .or(chosen_color)
                            .map(uno_ui_color)
                            .expect("反转牌应带有牌色或已选择的后续颜色"),
                    );
                }
                if play_index == 0
                    && let Some(color) = chosen_color
                {
                    spawn_uno_palette_effect(
                        &mut commands,
                        layer,
                        scene.discard,
                        color,
                        &mut palette_materials,
                    );
                }
            }
            UnoEvent::CardsDrawn {
                player,
                count,
                penalty,
                card_backs,
            } => {
                let interval = if !penalty && count > 1 { 0.18 } else { 0.045 };
                spawn_uno_draw_cards_with_interval(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(player),
                    count,
                    &card_backs,
                    0.0,
                    interval,
                    &game_assets,
                );
            }
            UnoEvent::StackNumberRevealed { cards, .. } => {
                for (index, card) in cards.into_iter().enumerate() {
                    let card_pose = uno_discard_pose(card);
                    let target = scene.discard_target(card);
                    spawn_uno_flying_card(
                        &mut commands,
                        scene.layer,
                        uno_card_handle(&game_assets, card),
                        Some(card),
                        scene.draw,
                        target,
                        UNO_PLAY_CARD_DURATION + 0.08 + index as f32 * 0.12,
                        true,
                        index,
                        card_pose.2,
                    );
                }
            }
            UnoEvent::CardsDiscarded { player, cards } => {
                let source = scene.player(player);
                for (index, card) in cards.into_iter().take(8).enumerate() {
                    let pose = uno_discard_pose(card);
                    spawn_uno_flying_card(
                        &mut commands,
                        scene.layer,
                        uno_card_handle(&game_assets, card),
                        Some(card),
                        source,
                        scene.discard + Vec2::new(pose.0, pose.1),
                        UNO_PLAY_CARD_DURATION * 0.35 + index as f32 * 0.045,
                        false,
                        index,
                        pose.2,
                    );
                }
            }
            UnoEvent::ColorRouletteResolved {
                player,
                color,
                count,
                card_backs,
            } => {
                spawn_uno_palette_effect(
                    &mut commands,
                    scene.layer,
                    scene.discard,
                    color,
                    &mut palette_materials,
                );
                spawn_uno_draw_cards_with_interval(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(player),
                    count,
                    &card_backs,
                    0.22,
                    0.18,
                    &game_assets,
                );
            }
            UnoEvent::DrawPenaltyReflected {
                target,
                count,
                card_backs,
                ..
            } => {
                spawn_uno_draw_cards_with_backs(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(target),
                    count,
                    &card_backs,
                    UNO_PLAY_CARD_DURATION * 0.72,
                    &game_assets,
                );
            }
            UnoEvent::ChallengeResolved {
                penalized,
                count,
                card_backs,
                ..
            } => {
                spawn_uno_draw_cards_with_backs(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(penalized),
                    count,
                    &card_backs,
                    0.18,
                    &game_assets,
                );
            }
            UnoEvent::SkipResolved {
                player,
                drew_card: true,
                card_back,
                ..
            } => {
                spawn_uno_draw_cards_with_backs(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(player),
                    1,
                    card_back.as_slice(),
                    0.02,
                    &game_assets,
                );
            }
            UnoEvent::UnoReported {
                target, card_backs, ..
            } => {
                spawn_uno_draw_cards_with_backs(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    scene.player(target),
                    2,
                    &card_backs,
                    0.12,
                    &game_assets,
                );
            }
            UnoEvent::ColorChosen { color, .. } => {
                spawn_uno_palette_effect(
                    &mut commands,
                    scene.layer,
                    scene.discard,
                    color,
                    &mut palette_materials,
                );
            }
            UnoEvent::HandRefreshed { player, count } => {
                let player_position = scene.player(player);
                let visible = count.min(6);
                spawn_uno_transfer_cards(
                    &mut commands,
                    scene.layer,
                    player_position,
                    scene.discard,
                    visible,
                    UNO_PLAY_CARD_DURATION * 0.72,
                    &game_assets,
                );
                spawn_uno_draw_cards(
                    &mut commands,
                    scene.layer,
                    scene.draw,
                    player_position,
                    visible,
                    UNO_PLAY_CARD_DURATION * 0.72 + 0.34,
                    &game_assets,
                );
            }
            UnoEvent::SwapOneCardTaken { player, target } => {
                let source = scene.player(target);
                let target = scene.player(player);
                spawn_uno_transfer_cards(
                    &mut commands,
                    scene.layer,
                    source,
                    target,
                    1,
                    0.0,
                    &game_assets,
                );
            }
            UnoEvent::SwapOneCompleted { player, target } => {
                let source = scene.player(player);
                let target = scene.player(target);
                spawn_uno_transfer_cards(
                    &mut commands,
                    scene.layer,
                    source,
                    target,
                    1,
                    0.0,
                    &game_assets,
                );
            }
            UnoEvent::HandsTraded { first, second, .. } => {
                let first_position = scene.player(first);
                let second_position = scene.player(second);
                spawn_uno_transfer_cards(
                    &mut commands,
                    scene.layer,
                    first_position,
                    second_position,
                    3,
                    0.0,
                    &game_assets,
                );
                spawn_uno_transfer_cards(
                    &mut commands,
                    scene.layer,
                    second_position,
                    first_position,
                    3,
                    0.08,
                    &game_assets,
                );
            }
            UnoEvent::HandsPassed { direction, .. } => {
                let mut ring = game
                    .players
                    .iter()
                    .filter(|player| !player.eliminated)
                    .collect::<Vec<_>>();
                ring.sort_by_key(|player| player.seat.0);
                let count = ring.len();
                if count < 2 {
                    continue;
                }
                for (index, player) in ring.iter().enumerate() {
                    let target_index = match direction {
                        UnoDirection::Clockwise => (index + 1) % count,
                        UnoDirection::CounterClockwise => (index + count - 1) % count,
                    };
                    let source = scene.player(player.id);
                    let target = scene.player(ring[target_index].id);
                    spawn_uno_transfer_cards(
                        &mut commands,
                        scene.layer,
                        source,
                        target,
                        2,
                        UNO_PLAY_CARD_DURATION * 0.72 + index as f32 * 0.035,
                        &game_assets,
                    );
                }
            }
            UnoEvent::Flipped { side } => {
                spawn_uno_flip_effect(
                    &mut commands,
                    scene.layer,
                    scene.layer_size,
                    previous_game.as_ref(),
                    side,
                    &flip_targets,
                    &ui_assets,
                    &game_assets,
                );
            }
            UnoEvent::UnoCalled { .. }
            | UnoEvent::SkipResolved {
                drew_card: false, ..
            }
            | UnoEvent::GameFinished { .. } => {}
        }
    }
    presentation.last_snapshot = Some(game.clone());
}
