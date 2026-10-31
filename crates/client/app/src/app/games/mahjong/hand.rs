use super::layout::MahjongLayout as Layout;
use super::tiles::{add_mahjong_hand_tile, mahjong_deal_spec, mahjong_draw_spec};
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_MELD_WIDTH, MahjongAssets, MahjongClaimHandShift,
    MahjongDiscardHandShift, MahjongOwnDiscardAnimation, MahjongTileHighlight, MahjongTileMaterial,
    MahjongTileSize, MahjongTileVisual, MahjongUiAction, MahjongWinningHand,
    add_mahjong_response_indicator, add_mahjong_tile_material, add_mahjong_wait_popup,
    apply_mahjong_winning_hand_visual, mahjong_claim_hand_shift_x, mahjong_discard_waits,
    mahjong_own_row_left, mahjong_win_tile_cues, mahjong_winning_hand_progress,
    mark_mahjong_win_tile,
};
use crate::app::presentation::GameSummaryAnimation;
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::picking::Pickable;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use leocard_ui::{UiComposer, spawn_node};

use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(super) struct MahjongWinningHandVisual {
    pub start: f32,
    pub reveal_duration: f32,
}

pub(super) struct MahjongOwnHandVisuals<'a> {
    pub response_tile: Option<MahjongTile>,
    pub show_wait_fans: bool,
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
    let meld_count = game
        .players
        .iter()
        .find(|player| player.player.id == game.you)
        .map_or(0, |player| player.melds.len());
    let can_discard =
        matches!(game.phase, MahjongPhaseView::Playing) && game.current_player == game.you;
    let discard_waits = can_discard.then(|| mahjong_discard_waits(game, show_wait_fans));
    let separated_tile = if dealing {
        game.your_drawn_tile
    } else if can_discard {
        game.your_drawn_tile.or_else(|| {
            (game.your_hand.len() % 3 == 2)
                .then(|| game.your_hand.last().copied())
                .flatten()
        })
    } else {
        None
    };
    let separated_index = separated_tile
        .and_then(|separated| game.your_hand.iter().position(|tile| *tile == separated));
    let regular_tile_count = game.your_hand.len() - usize::from(separated_index.is_some());
    let left = mahjong_own_row_left(meld_count, regular_tile_count)
        + meld_count as f32 * MAHJONG_OWN_MELD_WIDTH;
    let hand = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            width: px(Layout::OWN_ROW_WIDTH),
            bottom: px(8),
            height: px(Layout::OWN_ROW_HEIGHT),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        None,
    );
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
    for (index, tile) in game.your_hand.iter().copied().enumerate() {
        if separated_index == Some(index) {
            continue;
        }
        let new_x = left + regular_slot as f32 * Layout::OWN_STEP;
        regular_slot += 1;
        let result = match &game.phase {
            MahjongPhaseView::Finished { result } => Some(result),
            _ => None,
        };
        let cues = result.map(|result| {
            mahjong_win_tile_cues(result, |winner| {
                winner.winning_tile == tile
                    && (winner.from == Some(game.you)
                        || (winner.from.is_none() && winner.player == game.you))
            })
        });
        if winning_hand {
            let entity = add_mahjong_tile_material(
                commands,
                hand,
                MahjongTileVisual {
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
                can_discard.then_some(UiAction::Mahjong(MahjongUiAction::Discard(tile))),
                index,
                (dealing && !observed_hand.contains(&tile))
                    .then(|| mahjong_deal_spec(0, index, game.your_hand.len(), Layout::OWN_STEP)),
                assets,
                materials,
            );
            if response_tile == Some(tile) {
                add_mahjong_response_indicator(
                    commands,
                    entity,
                    Vec2::new(Layout::OWN_WIDTH, Layout::OWN_HEIGHT),
                    0.0,
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
        let tile = game.your_hand[index];
        let entity = add_mahjong_hand_tile(
            commands,
            hand,
            tile,
            can_discard.then_some(UiAction::Mahjong(MahjongUiAction::Discard(tile))),
            game.your_hand.len(),
            if drawn_tile_falling && game.your_drawn_tile == Some(tile) {
                Some(mahjong_draw_spec(0))
            } else {
                (dealing && !observed_hand.contains(&tile)).then(|| {
                    mahjong_deal_spec(
                        0,
                        game.your_hand.len().saturating_sub(1),
                        game.your_hand.len(),
                        Layout::OWN_STEP,
                    )
                })
            },
            assets,
            materials,
        );
        if response_tile == Some(tile) {
            add_mahjong_response_indicator(
                commands,
                entity,
                Vec2::new(Layout::OWN_WIDTH, Layout::OWN_HEIGHT),
                0.0,
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
