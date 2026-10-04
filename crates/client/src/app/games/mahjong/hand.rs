use super::tiles::{add_mahjong_hand_tile, mahjong_deal_spec, mahjong_draw_spec};
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_MELD_WIDTH, MahjongAssets, MahjongClaimHandShift,
    MahjongDiscardHandShift, MahjongHandTile, MahjongOwnDiscardAnimation, MahjongTileMaterial,
    MahjongTileSize, MahjongTileVisual, MahjongUiAction, MahjongWinningHand,
    add_mahjong_tile_material, add_mahjong_wait_popup, apply_mahjong_winning_hand_visual,
    mahjong_claim_hand_shift_x, mahjong_discard_waits, mahjong_own_row_left, mahjong_win_tile_cues,
    mahjong_winning_hand_progress, mark_mahjong_win_tile,
};
use crate::app::presentation::{GameSummaryAnimation, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::UiAction;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(super) struct MahjongWinningHandVisual {
    pub start: f32,
    pub reveal_duration: f32,
}

pub(super) struct MahjongOwnHandVisuals<'a> {
    pub observed_hand: &'a [MahjongTile],
    pub hover_lifts: &'a HashMap<i32, (f32, Interaction)>,
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
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    visuals: MahjongOwnHandVisuals<'_>,
) {
    let MahjongOwnHandVisuals {
        observed_hand,
        hover_lifts,
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
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.melds.len());
    let can_discard =
        matches!(game.phase, MahjongPhaseView::Playing) && game.current_player == game.you;
    let discard_waits = can_discard.then(|| mahjong_discard_waits(game));
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
            width: px(760),
            bottom: px(8),
            height: px(88),
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
        let new_x = left + regular_slot as f32 * 50.0;
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
                    highlighted: cues.as_ref().is_some_and(|cues| !cues.is_empty()),
                    deal: None,
                    relative: 0,
                },
                assets,
                materials,
            );
            if let (Some(result), Some(cues)) = (result, cues) {
                mark_mahjong_win_tile(commands, entity, result, cues);
            }
        } else {
            let entity = add_mahjong_hand_tile(
                commands,
                hand,
                tile.kind(),
                can_discard.then_some(UiAction::Mahjong(MahjongUiAction::Discard(tile))),
                index,
                (dealing && !observed_hand.contains(&tile))
                    .then(|| mahjong_deal_spec(0, index, game.your_hand.len(), 50.0)),
                assets,
                materials,
            );
            restore_hand_hover(
                commands,
                entity,
                hover_lifts.get(&(index as i32)).copied(),
                index as i32,
            );
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
                mark_mahjong_win_tile(commands, entity, result, cues);
            }
        }
    }
    if let Some(index) = separated_index {
        let gap = spawn_node(
            commands,
            hand,
            Node {
                width: px(22),
                min_width: px(22),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(FocusPolicy::Pass);
        let tile = game.your_hand[index];
        let entity = add_mahjong_hand_tile(
            commands,
            hand,
            tile.kind(),
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
                        50.0,
                    )
                })
            },
            assets,
            materials,
        );
        restore_hand_hover(
            commands,
            entity,
            hover_lifts.get(&(game.your_hand.len() as i32)).copied(),
            game.your_hand.len() as i32,
        );
        if let Some(waits) = discard_waits
            .as_ref()
            .and_then(|waits| waits.get(&tile.kind()))
        {
            add_mahjong_wait_popup(commands, entity, waits, assets, materials, ui_assets);
        }
    }
}

fn restore_hand_hover(
    commands: &mut Commands,
    entity: Entity,
    hover: Option<(f32, Interaction)>,
    index: i32,
) {
    let Some((lift, interaction)) = hover else {
        return;
    };
    if lift <= 0.0 {
        return;
    }
    commands.entity(entity).insert((
        MahjongHandTile {
            lift,
            base_rotation: 0.0,
            index,
        },
        // A rebuilt button must never inherit the previous button's click.
        if interaction == Interaction::Pressed {
            Interaction::Hovered
        } else {
            interaction
        },
        UiTransform {
            translation: Val2::px(0.0, -11.0 * lift),
            scale: Vec2::splat(1.0 + 0.025 * lift),
            ..default()
        },
        ZIndex(if lift > 0.05 { 100 + index } else { index }),
        BoxShadow::new(
            Color::BLACK.with_alpha(lift * 0.22),
            px(1),
            px(7.0 + lift * 6.0),
            px(0),
            px(3.0 + lift * 4.0),
        ),
    ));
}
