use super::tiles::{mahjong_deal_spec, mahjong_draw_spec};
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_REMOTE_DRAW_GAP, MAHJONG_REMOTE_MELD_WIDTH,
    MahjongAssets, MahjongClaimHandShift, MahjongDiscardRiverTile, MahjongOwnDiscardAnimation,
    MahjongRemoteDiscardAnimation, MahjongRemoteDiscardRiverTile, MahjongRemoteHandShift,
    MahjongTileHighlight, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    MahjongWinningHand, MahjongWinningHandVisual, add_mahjong_response_indicator,
    add_mahjong_tile_material, apply_mahjong_winning_hand_visual, mahjong_claim_hand_shift_x,
    mahjong_claim_landing_time, mahjong_local_light, mahjong_local_shadow, mahjong_own_row_left,
    mahjong_remote_tile_advance, mahjong_remote_tile_overhang, mahjong_win_tile_cues,
    mahjong_winning_hand_progress, mark_mahjong_win_tile, render_mahjong_meld,
    render_mahjong_staged_meld,
};
use crate::app::presentation::{
    DANGER, GameSummaryAnimation, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT,
    add_player_portrait, add_text, attach_start_game_seat_transition, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::SeatSide;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_mahjong::MahjongTile;
use leocard_protocol::{
    MahjongHandResultView, MahjongPhaseView, MahjongPlayerState, MahjongSnapshot, PlayerId,
};
use std::f32::consts;

pub(super) struct MahjongPlayerPanelVisuals<'a> {
    pub own_seat: u8,
    pub interaction_menu_open: Option<PlayerId>,
    pub start_transition_active: bool,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
}

pub(super) fn render_mahjong_player_panel(
    commands: &mut Commands,
    table: Entity,
    player: &MahjongPlayerState,
    visuals: MahjongPlayerPanelVisuals<'_>,
) {
    let MahjongPlayerPanelVisuals {
        own_seat,
        interaction_menu_open,
        start_transition_active,
        assets,
        avatars,
    } = visuals;
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    let (left, top, bottom) = match relative {
        0 => (20.0, None, Some(8.0)),
        1 => (1280.0 - 96.0 * 1.17 - 20.0, Some(284.0), None),
        // Keep the taller portrait beside the upper hand, not over its tiles.
        2 => (920.0, Some(8.0), None),
        _ => (20.0, Some(284.0), None),
    };
    let side = match relative {
        1 => SeatSide::Right,
        2 => SeatSide::Top,
        _ => SeatSide::Left,
    };
    let portrait = add_player_portrait(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: top.map_or(Val::Auto, px),
            bottom: bottom.map_or(Val::Auto, px),
            width: px(96.0 * 1.17),
            height: px(76.0 * 1.17 + 20.0),
            ..default()
        },
        PlayerPortraitSpec {
            player: player.id,
            profile: PlayerMenuProfile {
                name: &player.name,
                avatar: player.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: player.reference_points,
                completed_games: player.completed_games,
                game_profiles: &player.game_profiles,
            },
            side,
            avatar_size: 52.0 * 1.17,
            auto_play: player.auto_play,
            menu_open: interaction_menu_open == Some(player.id),
            menu_above: relative == 0,
            name_color: if player.dead_hand { DANGER } else { TEXT },
        },
        assets,
    );
    attach_start_game_seat_transition(
        commands,
        portrait.portrait,
        player.id,
        start_transition_active,
    );
    let flower_count = spawn_node(
        commands,
        portrait.portrait,
        Node {
            position_type: PositionType::Absolute,
            top: px(76.0 * 1.17 + 2.0),
            width: percent(100),
            height: px(18),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands.entity(flower_count).insert(Pickable::IGNORE);
    let label = add_text(
        commands,
        flower_count,
        format!("补花 {}", player.flowers.len()),
        13.0,
        MUTED,
        assets,
    );
    commands
        .entity(label)
        .insert((TextLayout::default().with_no_wrap(), Pickable::IGNORE));
}

pub(super) fn render_mahjong_wall(
    commands: &mut Commands,
    table: Entity,
    wall_len: u16,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let stack_count = usize::from(wall_len).div_ceil(2).min(72);
    for side in 0..4 {
        let count = stack_count.saturating_sub(side * 18).min(18);
        let (left, top, rotation) = match side {
            0 => (460.0, 105.0, 0.0),
            1 => (820.0, 302.0, consts::FRAC_PI_2),
            2 => (460.0, 500.0, consts::PI),
            _ => (100.0, 302.0, -consts::FRAC_PI_2),
        };
        let segment = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(360),
                height: px(46),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                ..default()
            },
            None,
        );
        commands.entity(segment).insert((
            UiTransform::from_rotation(Rot2::radians(rotation)),
            ZIndex(1),
        ));
        for index in 0..count {
            let wall_stack_index = side * 18 + index;
            let double = wall_stack_index * 2 + 1 < usize::from(wall_len);
            let layers = if double { 2 } else { 1 };
            let stack = spawn_node(
                commands,
                segment,
                Node {
                    width: px(20),
                    min_width: px(20),
                    height: px(44),
                    ..default()
                },
                None,
            );
            commands.entity(stack).insert(ZIndex(index as i32));
            let orientation = match side {
                1 => 3,
                3 => 1,
                _ => side as u8,
            };
            add_mahjong_wall_stack(commands, stack, layers, orientation, assets, materials);
        }
    }
}

fn add_mahjong_wall_stack(
    commands: &mut Commands,
    stack: Entity,
    layers: u8,
    orientation: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let material = materials.add(MahjongTileMaterial {
        params: Vec4::new(0.0, 1.0, 1.0, if layers == 2 { 2.0 } else { 3.0 }),
        lighting: mahjong_local_light(orientation),
        highlight: Vec4::ZERO,
        glyph: assets.tile_back.clone(),
        height: assets.tile_back.clone(),
    });
    let shadow = mahjong_local_shadow(orientation);
    let tile = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(26),
                height: px(46),
                border_radius: BorderRadius::all(px(3)),
                overflow: Overflow::visible(),
                ..default()
            },
            MaterialNode(material),
            BoxShadow::new(
                Color::BLACK.with_alpha(0.10),
                px(shadow.x),
                px(shadow.y),
                px(0),
                px(2),
            ),
            ZIndex(1),
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(stack).add_child(tile);
}

pub(super) struct MahjongPlayerTileVisuals<'a> {
    pub robbing_tile: Option<MahjongTile>,
    pub ui_assets: &'a UiAssets,
    pub own_seat: u8,
    pub observed_count: u8,
    pub flower_replaced: bool,
    pub dealing: bool,
    pub winning_hand: Option<MahjongWinningHandVisual>,
    pub separate_last_concealed: bool,
    pub drawn_tile_falling: bool,
    pub animation: &'a GameSummaryAnimation,
    pub active_claim: Option<&'a ActiveMahjongClaimPresentation>,
    pub remote_discard: Option<&'a MahjongRemoteDiscardAnimation>,
    pub result: Option<&'a MahjongHandResultView>,
    pub game_assets: &'a MahjongAssets,
    pub materials: &'a mut Assets<MahjongTileMaterial>,
}

pub(super) fn render_mahjong_player_tiles(
    commands: &mut Commands,
    table: Entity,
    player: &MahjongPlayerState,
    visuals: MahjongPlayerTileVisuals<'_>,
) {
    let MahjongPlayerTileVisuals {
        robbing_tile,
        ui_assets,
        own_seat,
        observed_count,
        flower_replaced,
        dealing,
        winning_hand,
        separate_last_concealed,
        drawn_tile_falling,
        animation,
        active_claim,
        remote_discard,
        result,
        game_assets,
        materials,
    } = visuals;
    let (winning_hand_start, win_reveal_duration) = winning_hand
        .map(|visual| (visual.start, visual.reveal_duration))
        .unwrap_or_default();
    let winning_hand = winning_hand.is_some();
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    let remote_discard =
        remote_discard.filter(|active| active.player == player.id && relative != 0);
    let separate_last_concealed = robbing_tile.is_some()
        || remote_discard
            .map(|active| active.from_drawn)
            .unwrap_or(separate_last_concealed);
    if relative == 0 && player.melds.is_empty() {
        return;
    }
    let (left, top, bottom, width, height, rotation) = match relative {
        0 => (
            mahjong_own_row_left(
                player.melds.len(),
                usize::from(player.concealed_count)
                    .saturating_sub(usize::from(separate_last_concealed)),
            ),
            None,
            Some(8.0),
            760.0,
            88.0,
            0.0,
        ),
        1 => (840.0, Some(302.0), None, 450.0, 54.0, -consts::FRAC_PI_2),
        2 => (415.0, Some(56.0), None, 450.0, 54.0, consts::PI),
        _ => (-10.0, Some(302.0), None, 450.0, 54.0, consts::FRAC_PI_2),
    };
    let group = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: top.map_or(Val::Auto, px),
            bottom: bottom.map_or(Val::Auto, px),
            width: px(width),
            height: px(height),
            align_items: AlignItems::FlexEnd,
            justify_content: if relative == 0 {
                JustifyContent::FlexStart
            } else {
                JustifyContent::Center
            },
            flex_direction: FlexDirection::Row,
            column_gap: px(0),
            ..default()
        },
        None,
    );
    let mut transform = UiTransform::from_rotation(Rot2::radians(rotation));
    if winning_hand {
        apply_mahjong_winning_hand_visual(
            &mut transform,
            relative,
            rotation,
            mahjong_winning_hand_progress(
                animation.elapsed,
                win_reveal_duration,
                winning_hand_start,
            ),
        );
        commands.entity(group).insert(MahjongWinningHand {
            relative,
            base_rotation: rotation,
            reveal_duration: win_reveal_duration,
            start: winning_hand_start,
        });
    }
    commands
        .entity(group)
        .insert((transform, ZIndex(if winning_hand { 100 } else { 8 })));

    let staged_claim = active_claim
        .filter(|claim| claim.source.is_some() && claim.elapsed < mahjong_claim_landing_time());
    let mut meld_index = 20;
    let visible_meld_count = player
        .melds
        .len()
        .saturating_sub(usize::from(staged_claim.is_some()));
    for meld in player.melds.iter().take(visible_meld_count) {
        render_mahjong_meld(
            commands,
            group,
            meld,
            &mut meld_index,
            relative,
            game_assets,
            materials,
        );
    }
    if let Some(claim) = staged_claim {
        render_mahjong_staged_meld(
            commands,
            group,
            claim,
            &mut meld_index,
            relative,
            game_assets,
            materials,
        );
    }
    if relative != 0 && (visible_meld_count > 0 || staged_claim.is_some()) {
        let gap = spawn_node(
            commands,
            group,
            Node {
                width: px(10),
                min_width: px(10),
                height: px(1),
                ..default()
            },
            None,
        );
        commands.entity(gap).insert(Pickable::IGNORE);
    }

    if relative != 0 {
        let concealed = spawn_node(
            commands,
            group,
            Node {
                height: px(height),
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::Row,
                flex_shrink: 0.0,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        if let Some(claim) = active_claim.filter(|claim| claim.shift_hand) {
            let distance = if player.melds.len() == 1 {
                MAHJONG_REMOTE_MELD_WIDTH + 10.0
            } else {
                MAHJONG_REMOTE_MELD_WIDTH
            };
            commands.entity(concealed).insert((
                MahjongClaimHandShift {
                    player: player.id,
                    distance,
                },
                UiTransform::from_translation(Val2::px(
                    mahjong_claim_hand_shift_x(claim.elapsed, distance),
                    0.0,
                )),
            ));
        }
        // The drawn tile sits outside the centered row. A discard from the hand
        // replaces its slot with that tile, so this width stays constant.
        let concealed_count = usize::from(player.concealed_count);
        let regular_count = concealed_count.saturating_sub(usize::from(
            separate_last_concealed && remote_discard.is_none(),
        ));
        let hidden_size = match relative {
            1 | 3 => MahjongTileSize::HiddenSide,
            _ => MahjongTileSize::HiddenOpposite,
        };
        if let Some(revealed) = &player.revealed_hand {
            for (index, tile) in revealed.iter().enumerate() {
                let cues = result.map(|result| {
                    mahjong_win_tile_cues(result, |winner| {
                        winner.winning_tile == *tile
                            && (winner.from == Some(player.id)
                                || (winner.from.is_none() && winner.player == player.id))
                    })
                });
                let entity = add_mahjong_tile_material(
                    commands,
                    concealed,
                    MahjongTileVisual {
                        kind: Some(tile.kind()),
                        size: if winning_hand {
                            MahjongTileSize::Mini
                        } else {
                            hidden_size
                        },
                        index,
                        highlight: MahjongTileHighlight::None,
                        deal: (dealing
                            && (index >= usize::from(observed_count)
                                || (flower_replaced && index + 1 == concealed_count)))
                            .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                        relative,
                    },
                    game_assets,
                    materials,
                );
                if robbing_tile == Some(*tile) {
                    add_mahjong_response_indicator(
                        commands,
                        entity,
                        if matches!(relative, 1 | 3) {
                            Vec2::new(34.0, 46.0)
                        } else {
                            Vec2::new(31.0, 42.0)
                        },
                        rotation,
                        ui_assets,
                    );
                }
                if let (Some(result), Some(cues)) = (result, cues) {
                    mark_mahjong_win_tile(
                        commands,
                        entity,
                        result,
                        cues,
                        winning_hand.then_some(winning_hand_start),
                    );
                }
            }
        } else {
            let advance = mahjong_remote_tile_advance(relative, false);
            let discarded_index = regular_count / 2;
            for index in 0..regular_count {
                let entity = add_mahjong_tile_material(
                    commands,
                    concealed,
                    MahjongTileVisual {
                        kind: None,
                        size: hidden_size,
                        index,
                        highlight: MahjongTileHighlight::None,
                        deal: (dealing
                            && (index >= usize::from(observed_count)
                                || (flower_replaced && index + 1 == concealed_count)))
                            .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                        relative,
                    },
                    game_assets,
                    materials,
                );
                if let Some(active) = remote_discard.filter(|active| !active.from_drawn) {
                    let start_x = if index == regular_count - 1 {
                        advance + MAHJONG_REMOTE_DRAW_GAP
                    } else if index >= discarded_index {
                        advance
                    } else {
                        0.0
                    };
                    if start_x > 0.0 {
                        commands.entity(entity).insert((
                            MahjongRemoteHandShift {
                                start_x,
                                end_x: 0.0,
                            },
                            active.hand_shift_transform(start_x, 0.0),
                        ));
                    }
                }
            }
            let ghost = remote_discard;
            if regular_count > 0 && (separate_last_concealed || ghost.is_some()) {
                let from_drawn = ghost.is_none_or(|active| active.from_drawn);
                let tile_left = if from_drawn {
                    regular_count as f32 * advance + MAHJONG_REMOTE_DRAW_GAP
                } else {
                    discarded_index as f32 * advance
                };
                let entity = add_mahjong_tile_material(
                    commands,
                    concealed,
                    MahjongTileVisual {
                        kind: ghost
                            .map(|active| active.tile.kind())
                            .or(robbing_tile.map(|tile| tile.kind())),
                        size: if ghost.is_some() || robbing_tile.is_some() {
                            MahjongTileSize::River
                        } else {
                            hidden_size
                        },
                        index: if from_drawn {
                            regular_count
                        } else {
                            discarded_index
                        },
                        highlight: MahjongTileHighlight::None,
                        deal: (ghost.is_none() && drawn_tile_falling)
                            .then(|| mahjong_draw_spec(relative)),
                        relative,
                    },
                    game_assets,
                    materials,
                );
                commands
                    .entity(entity)
                    .entry::<Node>()
                    .and_modify(move |mut node| {
                        node.position_type = PositionType::Absolute;
                        node.left = px(tile_left);
                        node.bottom = px(0);
                        node.margin = UiRect::ZERO;
                    });
                if robbing_tile.is_some() {
                    add_mahjong_response_indicator(
                        commands,
                        entity,
                        Vec2::new(33.0, 45.0),
                        rotation,
                        ui_assets,
                    );
                }
                if let Some(active) = ghost {
                    let (ghost, transform) = active.ghost_visual(0.0);
                    commands
                        .entity(entity)
                        .insert((ghost, transform, GlobalZIndex(900)));
                }
            }
        }
        if regular_count > 0 {
            let overhang = mahjong_remote_tile_overhang(
                relative,
                player.revealed_hand.is_some() && winning_hand,
            );
            let spacer = spawn_node(
                commands,
                concealed,
                Node {
                    width: px(overhang),
                    min_width: px(overhang),
                    height: px(1),
                    ..default()
                },
                None,
            );
            commands.entity(spacer).insert(Pickable::IGNORE);
        }
    }
}

pub(super) struct MahjongDiscardRiverVisuals<'a> {
    pub response_tile: Option<MahjongTile>,
    pub ui_assets: &'a UiAssets,
    pub discard_animation: Option<&'a MahjongOwnDiscardAnimation>,
    pub remote_discard: Option<&'a MahjongRemoteDiscardAnimation>,
}

pub(super) fn render_discard_rivers(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    animations: MahjongDiscardRiverVisuals<'_>,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let MahjongDiscardRiverVisuals {
        response_tile,
        ui_assets,
        discard_animation,
        remote_discard,
    } = animations;
    let result = match &game.phase {
        MahjongPhaseView::Finished { result } => Some(result),
        _ => None,
    };
    for player in &game.players {
        let relative = (player.seat.0 + 4 - own_seat) % 4;
        let (left, top, rotation) = match relative {
            0 => (549.0, 382.0, 0.0),
            1 => (735.0, 245.0, -consts::FRAC_PI_2),
            2 => (533.0, 150.0, consts::PI),
            _ => (331.0, 245.0, consts::FRAC_PI_2),
        };
        let river = spawn_node(
            commands,
            table,
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(182),
                height: px(126),
                align_content: AlignContent::FlexStart,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                row_gap: px(-3),
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        commands.entity(river).insert((
            UiTransform::from_rotation(Rot2::radians(rotation)),
            ZIndex(5),
        ));
        for (index, discard) in game
            .discards
            .iter()
            .enumerate()
            .filter(|(_, discard)| discard.player == player.id && discard.claimed_by.is_none())
        {
            let cues = result.map(|result| {
                mahjong_win_tile_cues(result, |winner| {
                    winner.from == Some(player.id) && winner.winning_tile == discard.tile
                })
            });
            let entity = add_mahjong_tile_material(
                commands,
                river,
                MahjongTileVisual {
                    kind: Some(discard.tile.kind()),
                    size: MahjongTileSize::River,
                    index,
                    highlight: if cues.as_ref().is_some_and(|cues| !cues.is_empty()) {
                        MahjongTileHighlight::Red
                    } else {
                        MahjongTileHighlight::None
                    },
                    deal: None,
                    relative,
                },
                assets,
                materials,
            );
            if response_tile == Some(discard.tile) {
                add_mahjong_response_indicator(
                    commands,
                    entity,
                    Vec2::new(33.0, 45.0),
                    rotation,
                    ui_assets,
                );
            }
            if discard_animation.is_some_and(|animation| animation.discard_index == index) {
                commands
                    .entity(entity)
                    .insert((MahjongDiscardRiverTile, Visibility::Hidden));
            }
            if remote_discard.is_some_and(|animation| animation.discard_index == index) {
                commands
                    .entity(entity)
                    .insert((MahjongRemoteDiscardRiverTile, Visibility::Hidden));
            }
            if let (Some(result), Some(cues)) = (result, cues) {
                mark_mahjong_win_tile(commands, entity, result, cues, None);
            }
        }
    }
}
