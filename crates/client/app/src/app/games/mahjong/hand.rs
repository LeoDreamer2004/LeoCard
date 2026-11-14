use super::layout::MahjongLayout as Layout;
use super::tiles::{
    MahjongHandTileInteraction, add_mahjong_hand_tile, mahjong_deal_spec, mahjong_draw_spec,
};
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_MELD_WIDTH, MahjongAssets, MahjongClaimHandShift,
    MahjongDiscardHandShift, MahjongDisplayedHand, MahjongOwnDiscardAnimation,
    MahjongReadyHandRegion, MahjongTileHighlight, MahjongTileMaterial, MahjongTileSize,
    MahjongTileVisual, MahjongUiAction, MahjongWinningHand, add_mahjong_ready_hand_tile,
    add_mahjong_response_indicator, add_mahjong_tile_material, add_mahjong_wait_popup,
    apply_mahjong_winning_hand_visual, mahjong_claim_hand_shift_x, mahjong_discard_waits,
    mahjong_own_row_left, mahjong_win_tile_cues, mahjong_winning_hand_progress,
    mark_mahjong_win_tile,
};
use crate::app::presentation::mahjong::own_overlay;
use crate::app::presentation::{GameSummaryAnimation, SilentButton};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy::ui_widgets::Button;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{UiComposer, spawn_node};

#[derive(Clone, Copy)]
pub(super) struct MahjongWinningHandVisual {
    pub start: f32,
    pub reveal_duration: f32,
}

pub(super) struct MahjongOwnHandVisuals<'a> {
    pub response_tile: Option<MahjongTile>,
    pub show_wait_fans: bool,
    pub declaring_ready: bool,
    pub observed_hand: &'a [MahjongTile],
    pub dealing: bool,
    pub drawn_tile_falling: bool,
    pub winning_hand: Option<MahjongWinningHandVisual>,
    pub animation: &'a GameSummaryAnimation,
    pub active_claim: Option<&'a ActiveMahjongClaimPresentation>,
    pub discard_animation: Option<&'a MahjongOwnDiscardAnimation>,
    pub assets: &'a MahjongAssets,
    pub ui_assets: &'a UiAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(super) fn render_own_hand(
    commands: &mut UiComposer,
    table: Entity,
    game: &MahjongSnapshot,
    visuals: MahjongOwnHandVisuals<'_>,
) {
    let MahjongOwnHandVisuals {
        response_tile,
        show_wait_fans,
        declaring_ready,
        observed_hand,
        dealing,
        drawn_tile_falling,
        winning_hand,
        animation,
        active_claim,
        discard_animation,
        assets,
        ui_assets,
        materials,
    } = visuals;
    let (winning_hand_start, win_reveal_duration) = winning_hand
        .map(|visual| (visual.start, visual.reveal_duration))
        .unwrap_or_default();
    let winning_hand = winning_hand.is_some();
    let face_down = game.your_declared_ready() && !winning_hand;
    let meld_count = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .map_or(0, |player| player.melds.len());
    let result = match &game.phase {
        MahjongPhaseView::Finished { result } => Some(result),
        _ => None,
    };
    let winner = result.and_then(|result| {
        result
            .winners
            .iter()
            .find(|winner| winner.player == game.you)
    });
    let displayed = MahjongDisplayedHand::new(&game.your_hand, winner);
    let winning_drawn = displayed.drawn;
    let mut hand_tiles = displayed.regular;
    hand_tiles.extend(winning_drawn);
    let can_discard =
        matches!(game.phase, MahjongPhaseView::Playing) && game.current_player == game.you;
    let discard_action = |tile| {
        if !can_discard {
            return None;
        }
        if declaring_ready {
            game.ready_discards
                .contains(&tile)
                .then_some(UiAction::Mahjong(MahjongUiAction::DeclareReady(tile)))
        } else if !game.your_declared_ready() || game.your_drawn_tile == Some(tile) {
            Some(UiAction::Mahjong(MahjongUiAction::Discard(tile)))
        } else {
            None
        }
    };
    let discard_waits = can_discard.then(|| mahjong_discard_waits(game, show_wait_fans));
    let tile_interaction = |tile| MahjongHandTileInteraction {
        action: discard_action(tile),
        dimmed: declaring_ready && !game.ready_discards.contains(&tile),
    };
    let separated_tile = if winner.is_some() {
        winning_drawn
    } else if dealing {
        game.your_drawn_tile
    } else if can_discard {
        game.your_drawn_tile.or_else(|| {
            (hand_tiles.len() % 3 == 2)
                .then(|| hand_tiles.last().copied())
                .flatten()
        })
    } else {
        None
    };
    let separated_index =
        separated_tile.and_then(|separated| hand_tiles.iter().position(|tile| *tile == separated));
    let regular_tile_count = hand_tiles.len() - usize::from(separated_index.is_some());
    let left = mahjong_own_row_left(meld_count, regular_tile_count)
        + meld_count as f32 * MAHJONG_OWN_MELD_WIDTH;
    let overlay = own_overlay(commands, table);
    let hand = spawn_node(
        commands,
        overlay,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            width: px(if face_down {
                regular_tile_count as f32 * Layout::OWN_STEP
                    + if separated_index.is_some() {
                        Layout::DRAW_GAP + Layout::OWN_WIDTH
                    } else {
                        0.0
                    }
            } else {
                Layout::OWN_ROW_WIDTH
            }),
            bottom: px(Layout::OWN_BOTTOM
                + if winning_hand || face_down {
                    0.0
                } else {
                    Layout::OWN_HAND_LIFT
                }),
            height: px(Layout::OWN_ROW_HEIGHT),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        None,
    );
    if face_down {
        commands
            .entity(hand)
            .insert((MahjongReadyHandRegion, Pickable::default()))
            .entry::<PickingInteraction>()
            .or_insert(PickingInteraction::None);
        if cfg!(target_os = "android") {
            commands.entity(hand).insert((
                Button,
                SilentButton,
                UiAction::Mahjong(MahjongUiAction::ToggleReadyHandPeek),
            ));
        }
    }
    if let Some(claim) = active_claim {
        commands.entity(hand).insert((
            MahjongClaimHandShift {
                player: game.you,
                distance: MAHJONG_OWN_MELD_WIDTH,
            },
            UiTransform::from_translation(Val2::px(
                mahjong_claim_hand_shift_x(claim.elapsed, MAHJONG_OWN_MELD_WIDTH),
                0.0,
            )),
        ));
    }
    if winning_hand {
        let progress = mahjong_winning_hand_progress(
            animation.elapsed,
            win_reveal_duration,
            winning_hand_start,
        );
        let mut transform = UiTransform::default();
        apply_mahjong_winning_hand_visual(&mut transform, 0, 0.0, progress);
        commands.entity(hand).insert((
            MahjongWinningHand {
                relative: 0,
                base_rotation: 0.0,
                reveal_duration: win_reveal_duration,
                start: winning_hand_start,
            },
            transform,
            ZIndex(100),
        ));
    }
    let mut regular_slot = 0;
    for (index, tile) in hand_tiles.iter().copied().enumerate() {
        if separated_index == Some(index) {
            continue;
        }
        let new_x = left + regular_slot as f32 * Layout::OWN_STEP;
        regular_slot += 1;
        let cues = result.map(|result| {
            mahjong_win_tile_cues(result, |winner| {
                winner.winning_tile == tile
                    && (winner.from == Some(game.you)
                        || (winner.from.is_none() && winner.player == game.you))
            })
        });
        if face_down {
            add_mahjong_ready_hand_tile(commands, hand, tile, index, assets, materials);
        } else if winning_hand {
            let entity = add_mahjong_tile_material(
                commands,
                hand,
                MahjongTileVisual {
                    elevation: 0.0,
                    kind: Some(tile.kind()),
                    size: MahjongTileSize::OwnMeld,
                    index,
                    highlight: MahjongTileHighlight::None,
                    deal: None,
                    relative: 0,
                },
                assets,
                materials,
            );
            if let (Some(result), Some(cues)) = (result, cues) {
                mark_mahjong_win_tile(commands, entity, result, cues, Some(winning_hand_start));
            }
        } else {
            let entity = add_mahjong_hand_tile(
                commands,
                hand,
                tile,
                tile_interaction(tile),
                index,
                (dealing && !observed_hand.contains(&tile))
                    .then(|| mahjong_deal_spec(0, index, hand_tiles.len(), Layout::OWN_STEP)),
                assets,
                materials,
            );
            if response_tile == Some(tile) {
                add_mahjong_response_indicator(
                    commands,
                    entity,
                    Vec2::new(Layout::OWN_WIDTH, Layout::OWN_HEIGHT),
                    0.0,
                    false,
                    ui_assets,
                );
            }
            if let Some(animation) = discard_animation
                && let Some(old_x) = animation.old_tile_x(tile)
            {
                let start_x = old_x - new_x;
                commands.entity(entity).insert((
                    MahjongDiscardHandShift {
                        start_x,
                        drawn: animation.old_drawn == Some(tile),
                    },
                    animation.hand_shift_transform(start_x),
                ));
            }
            if let Some(waits) = discard_waits
                .as_ref()
                .and_then(|waits| waits.get(&tile.kind()))
            {
                add_mahjong_wait_popup(commands, entity, waits, assets, materials, ui_assets);
            }
            if let (Some(result), Some(cues)) = (result, cues) {
                mark_mahjong_win_tile(commands, entity, result, cues, None);
            }
        }
    }
    if let Some(index) = separated_index {
        let gap = spawn_node(
            commands,
            hand,
            Node {
                width: px(Layout::DRAW_GAP),
                min_width: px(Layout::DRAW_GAP),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(Pickable::IGNORE);
        let tile = hand_tiles[index];
        let entity = if winning_hand || face_down {
            add_mahjong_tile_material(
                commands,
                hand,
                MahjongTileVisual {
                    elevation: 0.0,
                    kind: Some(tile.kind()),
                    size: if face_down {
                        MahjongTileSize::OwnHand
                    } else {
                        MahjongTileSize::OwnMeld
                    },
                    index: hand_tiles.len(),
                    highlight: MahjongTileHighlight::None,
                    deal: (face_down && drawn_tile_falling).then(|| mahjong_draw_spec(0)),
                    relative: 0,
                },
                assets,
                materials,
            )
        } else {
            add_mahjong_hand_tile(
                commands,
                hand,
                tile,
                tile_interaction(tile),
                hand_tiles.len(),
                if drawn_tile_falling && game.your_drawn_tile == Some(tile) {
                    Some(mahjong_draw_spec(0))
                } else {
                    (dealing && !observed_hand.contains(&tile)).then(|| {
                        mahjong_deal_spec(
                            0,
                            hand_tiles.len().saturating_sub(1),
                            hand_tiles.len(),
                            Layout::OWN_STEP,
                        )
                    })
                },
                assets,
                materials,
            )
        };
        if let Some(result) = result {
            let cues = mahjong_win_tile_cues(result, |winner| {
                winner.player == game.you && winner.from.is_none() && winner.winning_tile == tile
            });
            mark_mahjong_win_tile(
                commands,
                entity,
                result,
                cues,
                winning_hand.then_some(winning_hand_start),
            );
        }
        if response_tile == Some(tile) {
            add_mahjong_response_indicator(
                commands,
                entity,
                Vec2::new(Layout::OWN_WIDTH, Layout::OWN_HEIGHT),
                0.0,
                false,
                ui_assets,
            );
        }
        if let Some(waits) = discard_waits
            .as_ref()
            .and_then(|waits| waits.get(&tile.kind()))
        {
            add_mahjong_wait_popup(commands, entity, waits, assets, materials, ui_assets);
        }
    }
}
