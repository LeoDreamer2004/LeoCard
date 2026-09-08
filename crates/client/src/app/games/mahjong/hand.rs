use super::*;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{MahjongPhaseView, MahjongSnapshot};

#[derive(Clone, Copy)]
pub(super) struct MahjongWinningHandVisual {
    pub start: f32,
    pub reveal_duration: f32,
}

pub(super) struct MahjongOwnHandVisuals<'a> {
    pub observed_hand: &'a [MahjongTile],
    pub dealing: bool,
    pub winning_hand: Option<MahjongWinningHandVisual>,
    pub animation: &'a GameSummaryAnimation,
    pub active_claim: Option<&'a ActiveMahjongClaimPresentation>,
    pub assets: &'a UiAssets,
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
        dealing,
        winning_hand,
        animation,
        active_claim,
        assets,
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
    let left = MAHJONG_OWN_HAND_LEFT + meld_count as f32 * MAHJONG_OWN_MELD_WIDTH;
    let hand = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            width: px(760),
            bottom: px(8),
            height: px(80),
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
    let can_discard =
        matches!(game.phase, MahjongPhaseView::Playing) && game.current_player == game.you;
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
    for (index, tile) in game.your_hand.iter().copied().enumerate() {
        if separated_index == Some(index) {
            continue;
        }
        if winning_hand {
            add_mahjong_tile_material(
                commands,
                hand,
                MahjongTileVisual {
                    kind: Some(tile.kind()),
                    size: MahjongTileSize::OwnMeld,
                    index,
                    highlighted: false,
                    deal: None,
                    relative: 0,
                },
                assets,
                materials,
            );
        } else {
            add_mahjong_hand_tile(
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
        }
    }
    if let Some(index) = separated_index {
        let gap = spawn_node(
            commands,
            hand,
            Node {
                width: px(18),
                min_width: px(18),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(FocusPolicy::Pass);
        let tile = game.your_hand[index];
        add_mahjong_hand_tile(
            commands,
            hand,
            tile.kind(),
            can_discard.then_some(UiAction::Mahjong(MahjongUiAction::Discard(tile))),
            game.your_hand.len(),
            (dealing && !observed_hand.contains(&tile)).then(|| {
                mahjong_deal_spec(
                    0,
                    game.your_hand.len().saturating_sub(1),
                    game.your_hand.len(),
                    50.0,
                )
            }),
            assets,
            materials,
        );
    }
}
