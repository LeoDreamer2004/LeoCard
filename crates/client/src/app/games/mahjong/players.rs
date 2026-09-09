use super::tiles::mahjong_deal_spec;
use super::{
    ActiveMahjongClaimPresentation, MAHJONG_OWN_HAND_LEFT, MAHJONG_REMOTE_MELD_WIDTH,
    MahjongAssets, MahjongClaimHandShift, MahjongTileMaterial, MahjongTileSize, MahjongTileVisual,
    MahjongWinningHand, MahjongWinningHandVisual, add_mahjong_tile_material,
    apply_mahjong_winning_hand_visual, mahjong_claim_hand_shift_x, mahjong_claim_landing_time,
    mahjong_local_light, mahjong_local_shadow, mahjong_winning_hand_progress, render_mahjong_meld,
    render_mahjong_staged_meld, wind_label,
};
use crate::app::presentation::{
    BORDER, DANGER, GameSummaryAnimation, MUTED, PANEL, PlayerMenuProfile, TEXT, add_avatar,
    add_interaction_menu, add_text, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{OpponentBadge, PlayerAvatarAnchor, SeatSide, SocialUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::MahjongPlayerState;
use leocard_protocol::{MahjongSnapshot, PlayerId};

pub(super) struct MahjongPlayerPanelVisuals<'a> {
    pub own_seat: u8,
    pub interaction_menu_open: Option<PlayerId>,
    pub assets: &'a UiAssets,
    pub avatars: &'a AvatarImages,
}

pub(super) fn render_mahjong_player_panel(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    player: &MahjongPlayerState,
    visuals: MahjongPlayerPanelVisuals<'_>,
) {
    let MahjongPlayerPanelVisuals {
        own_seat,
        interaction_menu_open,
        assets,
        avatars,
    } = visuals;
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    let (left, top, bottom, width) = match relative {
        0 => (8.0, None, Some(8.0), 165.0),
        1 => (1107.0, Some(300.0), None, 165.0),
        2 => (557.5, Some(8.0), None, 165.0),
        _ => (8.0, Some(300.0), None, 165.0),
    };
    let panel = commands
        .spawn((
            Button,
            UiAction::Social(SocialUiAction::ToggleInteractionMenu(player.id)),
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: top.map_or(Val::Auto, px),
                bottom: bottom.map_or(Val::Auto, px),
                width: px(width),
                height: px(50),
                padding: UiRect::all(px(6)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(8)),
                ..default()
            },
            BackgroundColor(PANEL.with_alpha(0.92)),
            BorderColor::all(BORDER),
            BoxShadow::new(Color::BLACK.with_alpha(0.32), px(0), px(3), px(0), px(7)),
        ))
        .id();
    commands.entity(table).add_child(panel);
    let head = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            column_gap: px(7),
            ..default()
        },
        None,
    );
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar_entity = add_avatar(commands, head, &player.name, avatar, 30.0, assets);
    commands
        .entity(avatar_entity)
        .insert(PlayerAvatarAnchor(player.id));
    let info = spawn_node(
        commands,
        head,
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    add_text(
        commands,
        info,
        format!(
            "{}  {}{}",
            player.name,
            wind_label(player.seat_wind),
            if player.id == game.dealer { "庄" } else { "" }
        ),
        14.0,
        if player.dead_hand { DANGER } else { TEXT },
        assets,
    );
    add_text(
        commands,
        info,
        format!(
            "花 {} · 累计 {:+}",
            player.flowers.len(),
            game.match_scores[player.id.0 as usize]
        ),
        10.0,
        MUTED,
        assets,
    );
    let side = match relative {
        1 => SeatSide::Right,
        2 => SeatSide::Top,
        _ => SeatSide::Left,
    };
    let menu = add_interaction_menu(
        commands,
        panel,
        player.id,
        side,
        PlayerMenuProfile {
            name: &player.name,
            avatar,
            reference_points: player.reference_points,
            completed_games: player.completed_games,
            game_profiles: &player.game_profiles,
        },
        assets,
    );
    commands
        .entity(menu)
        .insert(if interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: None,
        interaction_menu: menu,
    });
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
            0 => (460.0, 135.0, 0.0),
            1 => (820.0, 302.0, std::f32::consts::FRAC_PI_2),
            2 => (460.0, 480.0, std::f32::consts::PI),
            _ => (100.0, 302.0, -std::f32::consts::FRAC_PI_2),
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
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(stack).add_child(tile);
}

pub(super) struct MahjongPlayerTileVisuals<'a> {
    pub own_seat: u8,
    pub observed_count: u8,
    pub flower_replaced: bool,
    pub dealing: bool,
    pub winning_hand: Option<MahjongWinningHandVisual>,
    pub separate_last_concealed: bool,
    pub animation: &'a GameSummaryAnimation,
    pub active_claim: Option<&'a ActiveMahjongClaimPresentation>,
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
        own_seat,
        observed_count,
        flower_replaced,
        dealing,
        winning_hand,
        separate_last_concealed,
        animation,
        active_claim,
        game_assets,
        materials,
    } = visuals;
    let (winning_hand_start, win_reveal_duration) = winning_hand
        .map(|visual| (visual.start, visual.reveal_duration))
        .unwrap_or_default();
    let winning_hand = winning_hand.is_some();
    let relative = (player.seat.0 + 4 - own_seat) % 4;
    if relative == 0 && player.melds.is_empty() {
        return;
    }
    let (left, top, bottom, width, height, rotation) = match relative {
        0 => (MAHJONG_OWN_HAND_LEFT, None, Some(8.0), 760.0, 80.0, 0.0),
        1 => (
            840.0,
            Some(302.0),
            None,
            450.0,
            54.0,
            -std::f32::consts::FRAC_PI_2,
        ),
        2 => (415.0, Some(58.0), None, 450.0, 54.0, std::f32::consts::PI),
        _ => (
            -10.0,
            Some(302.0),
            None,
            450.0,
            54.0,
            std::f32::consts::FRAC_PI_2,
        ),
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
            justify_content: JustifyContent::FlexStart,
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
        commands.entity(gap).insert(FocusPolicy::Pass);
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
        let concealed_count = usize::from(player.concealed_count);
        let hidden_size = match relative {
            1 | 3 => MahjongTileSize::HiddenSide,
            _ => MahjongTileSize::HiddenOpposite,
        };
        if let Some(revealed) = &player.revealed_hand {
            for (index, tile) in revealed.iter().enumerate() {
                add_mahjong_tile_material(
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
                        highlighted: false,
                        deal: (dealing
                            && (index >= usize::from(observed_count)
                                || (flower_replaced && index + 1 == concealed_count)))
                            .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                        relative,
                    },
                    game_assets,
                    materials,
                );
            }
        } else {
            let joined_count = concealed_count.saturating_sub(usize::from(separate_last_concealed));
            for index in 0..joined_count {
                add_mahjong_tile_material(
                    commands,
                    concealed,
                    MahjongTileVisual {
                        kind: None,
                        size: hidden_size,
                        index,
                        highlighted: false,
                        deal: (dealing
                            && (index >= usize::from(observed_count)
                                || (flower_replaced && index + 1 == concealed_count)))
                            .then(|| mahjong_deal_spec(relative, index, concealed_count, 24.0)),
                        relative,
                    },
                    game_assets,
                    materials,
                );
            }
            if separate_last_concealed && concealed_count > 0 {
                let gap = spawn_node(
                    commands,
                    concealed,
                    Node {
                        width: px(14),
                        min_width: px(14),
                        height: px(1),
                        ..default()
                    },
                    None,
                );
                commands.entity(gap).insert(FocusPolicy::Pass);
                add_mahjong_tile_material(
                    commands,
                    concealed,
                    MahjongTileVisual {
                        kind: None,
                        size: hidden_size,
                        index: concealed_count - 1,
                        highlighted: false,
                        deal: None,
                        relative,
                    },
                    game_assets,
                    materials,
                );
            }
        }
    }
}

pub(super) fn render_discard_rivers(
    commands: &mut Commands,
    table: Entity,
    game: &MahjongSnapshot,
    own_seat: u8,
    assets: &MahjongAssets,
    materials: &mut Assets<MahjongTileMaterial>,
) {
    let last_discard = game
        .discards
        .iter()
        .rposition(|discard| discard.claimed_by.is_none());
    for player in &game.players {
        let relative = (player.seat.0 + 4 - own_seat) % 4;
        let (left, top, rotation) = match relative {
            0 => (549.0, 382.0, 0.0),
            1 => (785.0, 245.0, -std::f32::consts::FRAC_PI_2),
            2 => (533.0, 160.0, std::f32::consts::PI),
            _ => (281.0, 245.0, std::f32::consts::FRAC_PI_2),
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
            add_mahjong_tile_material(
                commands,
                river,
                MahjongTileVisual {
                    kind: Some(discard.tile.kind()),
                    size: MahjongTileSize::River,
                    index,
                    highlighted: last_discard == Some(index),
                    deal: None,
                    relative,
                },
                assets,
                materials,
            );
        }
    }
}
