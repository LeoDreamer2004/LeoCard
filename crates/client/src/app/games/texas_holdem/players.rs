use super::{
    TexasChipTableState, TexasHoldemAssets, TexasPlayerPanel, TexasPlayerShake,
    texas_player_border_color, texas_player_chip_zone,
};
use crate::app::presentation::{
    ACCENT, HEADER_BG, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, TurnBorderAnimationKey,
    TurnBorderMaterial, add_player_portrait, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, position_opponent_popup, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{OpponentBadge, SeatSide};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_protocol::{GameKind, PlayerId, TexasHoldemPlayerState, TexasHoldemSnapshot};

pub(super) const TEXAS_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(super) const TEXAS_PORTRAIT_HEIGHT: f32 = 76.0 * 1.17 + 34.0;

#[expect(
    clippy::too_many_arguments,
    reason = "the opponent builder keeps seat state and visual resources explicit"
)]
pub(super) fn add_texas_opponent(
    commands: &mut Commands,
    table: Entity,
    player: &TexasHoldemPlayerState,
    relative: u8,
    game: &TexasHoldemSnapshot,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
    intro_only: bool,
) {
    let side = match relative {
        1 | 2 => SeatSide::Left,
        3 => SeatSide::Top,
        4 | 5 => SeatSide::Right,
        _ => return,
    };
    let zone = texas_player_chip_zone(relative);
    let left = match side {
        SeatSide::Left => zone.left - TEXAS_PORTRAIT_WIDTH - 8.0,
        SeatSide::Top | SeatSide::Right => zone.left + zone.width + 8.0,
    };
    let top = if matches!(side, SeatSide::Top) {
        zone.top + (zone.height - TEXAS_PORTRAIT_HEIGHT) * 0.5 - 16.0
    } else {
        zone.top + (zone.height - TEXAS_PORTRAIT_HEIGHT) * 0.5
    };
    let seat = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: px(top),
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(TEXAS_PORTRAIT_HEIGHT),
            ..default()
        },
        None,
    );
    commands.entity(seat).insert(TexasPlayerShake(player.id));
    attach_start_game_seat_transition(commands, seat, player.id, start_transition_active);
    let portrait = add_player_portrait(
        commands,
        seat,
        Node {
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(TEXAS_PORTRAIT_HEIGHT),
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
            menu_above: false,
            name_color: if player.folded { MUTED } else { TEXT },
        },
        assets,
    );
    let base_border = texas_player_border_color(player, game.current_player == Some(player.id));
    commands
        .entity(portrait.avatar_ring)
        .entry::<Node>()
        .and_modify(|mut node| node.border_radius = BorderRadius::all(px(52.0 * 1.17 * 0.2)));
    commands.entity(portrait.avatar_ring).insert((
        TexasPlayerPanel {
            player: player.id,
            base_border,
        },
        Outline::new(px(3.0), px(0), base_border),
        BoxShadow::new(Color::NONE, px(0), px(0), px(0), px(0)),
    ));
    if !player.folded && game.current_player == Some(player.id) {
        add_turn_border_trace_with_radius(
            commands,
            portrait.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::TexasHoldem, game.match_id, player.id),
            52.0 * 1.17 * 0.2,
            52.0 * 1.17,
        );
    }
    add_role_tokens(commands, portrait.avatar_ring, player.id, game, assets);
    add_texas_stack_value(
        commands,
        portrait.portrait,
        player.stack,
        assets,
        game_assets,
    );
    if intro_only {
        return;
    }

    let popup = add_texas_chip_popup(
        commands,
        seat,
        &player.name,
        player.stack,
        Some(side),
        assets,
        game_assets,
        &chip_state.stack_counts(player.id),
    );
    commands.entity(popup).insert(Visibility::Hidden);
    commands
        .entity(portrait.portrait)
        .entry::<OpponentBadge>()
        .and_modify(move |mut badge| {
            badge.score_popup = Some(popup);
        });
}

pub(super) fn add_texas_stack_value(
    commands: &mut Commands,
    portrait: Entity,
    stack: u32,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
) {
    let node = Node {
        position_type: PositionType::Absolute,
        top: px(76.0 * 1.17 + 2.0),
        width: percent(100),
        height: px(31),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        column_gap: px(8),
        ..default()
    };
    let area = spawn_node(commands, portrait, node, None);
    commands.entity(area).insert(FocusPolicy::Pass);
    let fan = spawn_node(
        commands,
        area,
        Node {
            width: px(35),
            height: px(30),
            position_type: PositionType::Relative,
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    commands.entity(fan).insert(FocusPolicy::Pass);
    for (denomination, left, top, rotation, color, layer) in [
        (1, 1.0, 6.0, -0.22, Color::BLACK, 0),
        (5, 12.0, 1.0, 0.16, Color::WHITE, 1),
    ] {
        let Some(image) = game_assets.poker_chips.get(&denomination) else {
            continue;
        };
        let chip = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(24),
                    height: px(24),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode::new(image.clone()),
                UiTransform::from_rotation(Rot2::radians(rotation)),
                ZIndex(layer),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(fan).add_child(chip);
        let label = add_text(commands, chip, denomination.to_string(), 9.0, color, assets);
        commands.entity(label).insert(FocusPolicy::Pass);
    }
    let digits = stack.to_string();
    let font_size = (28.0 - digits.len().saturating_sub(3) as f32 * 2.5).max(17.0);
    let value = add_text(commands, area, digits, font_size, ACCENT, assets);
    commands.entity(value).insert((
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        },
        FocusPolicy::Pass,
    ));
}

pub(super) fn add_role_tokens(
    commands: &mut Commands,
    avatar: Entity,
    player: PlayerId,
    game: &TexasHoldemSnapshot,
    assets: &UiAssets,
) {
    let mut labels = Vec::new();
    if player == game.dealer {
        labels.push("D");
    }
    if player == game.small_blind {
        labels.push("SB");
    }
    if player == game.big_blind {
        labels.push("BB");
    }
    if labels.is_empty() {
        return;
    }
    let row = spawn_node(
        commands,
        avatar,
        Node {
            position_type: PositionType::Absolute,
            right: px(-9),
            bottom: px(-7),
            flex_direction: FlexDirection::Row,
            column_gap: px(2),
            ..default()
        },
        None,
    );
    commands.entity(row).insert((ZIndex(45), FocusPolicy::Pass));
    for label in labels {
        let chip = spawn_node(
            commands,
            row,
            Node {
                min_width: px(18),
                height: px(18),
                padding: UiRect::horizontal(px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(chip).insert(FocusPolicy::Pass);
        add_text(commands, chip, label, 8.0, HEADER_BG, assets);
    }
}

/// 德州筹码面板沿用七鬼五二三分牌面板的外观，并显示账本中真实存在的筹码。
#[expect(
    clippy::too_many_arguments,
    reason = "the chip popup builder keeps player and stack presentation inputs explicit"
)]
pub(super) fn add_texas_chip_popup(
    commands: &mut Commands,
    parent: Entity,
    player_name: &str,
    stack: u32,
    opponent_side: Option<SeatSide>,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
    stack_counts: &[(u16, usize)],
) -> Entity {
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(3),
        ..default()
    };
    if let Some(side) = opponent_side {
        node.border = UiRect::all(px(1));
        node.border_radius = BorderRadius::all(px(8));
        position_opponent_popup(&mut node, side);
        node.top = px(TEXAS_PORTRAIT_HEIGHT + 6.0);
    } else {
        node.left = px(TEXAS_PORTRAIT_WIDTH + 8.0);
        node.top = px((76.0 * 1.17 - 66.0) * 0.5);
        node.width = px(300);
        node.min_height = px(66);
        node.padding = UiRect::new(px(76), px(6), px(6), px(6));
        node.justify_content = JustifyContent::Center;
    }
    let popup = spawn_node(
        commands,
        parent,
        node,
        opponent_side.map(|_| Color::BLACK.with_alpha(0.78)),
    );
    commands
        .entity(popup)
        .insert((GlobalZIndex(1500), FocusPolicy::Pass));
    if opponent_side.is_some() {
        commands
            .entity(popup)
            .insert(BorderColor::all(ACCENT.with_alpha(0.72)));
    } else {
        let mut image = ImageNode::new(assets.home.game_card.clone()).with_mode(
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect::all(16.0),
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            }),
        );
        image.visual_box = VisualBox::BorderBox;
        commands.entity(popup).insert(image);
    }

    if opponent_side.is_some() {
        add_text(
            commands,
            popup,
            format!("{player_name} 的筹码 · 剩余 {stack}"),
            12.0,
            ACCENT,
            assets,
        );
    } else {
        let value_area = spawn_node(
            commands,
            popup,
            Node {
                position_type: PositionType::Absolute,
                left: px(6),
                top: px(4),
                bottom: px(4),
                width: px(65),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(-2),
                ..default()
            },
            None,
        );
        commands.entity(value_area).insert(FocusPolicy::Pass);
        add_text(commands, value_area, "剩余", 10.0, MUTED, assets);
        let digits = stack.to_string();
        let font_size = (28.0 - digits.len().saturating_sub(3) as f32 * 2.5).max(17.0);
        let value = add_text(commands, value_area, digits, font_size, ACCENT, assets);
        commands.entity(value).insert(TextShadow {
            offset: Vec2::new(1.5, 2.0),
            color: Color::BLACK.with_alpha(0.82),
        });
    }

    let chips = spawn_node(
        commands,
        popup,
        Node {
            width: percent(100),
            height: px(38),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(if opponent_side.is_some() { 18 } else { 10 }),
            ..default()
        },
        None,
    );
    for &(denomination, count) in stack_counts {
        add_horizontal_chip_group(commands, chips, denomination, count, assets, game_assets);
    }
    popup
}

/// 同面值筹码水平紧叠，不同面值由父节点的 column_gap 分组隔开。
fn add_horizontal_chip_group(
    commands: &mut Commands,
    parent: Entity,
    denomination: u16,
    count: usize,
    assets: &UiAssets,
    game_assets: &TexasHoldemAssets,
) {
    const CHIP_SIZE: f32 = 34.0;
    const CHIP_REVEAL: f32 = 11.0;
    let group_width = CHIP_SIZE + count.saturating_sub(1) as f32 * CHIP_REVEAL;
    let group = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Relative,
            width: px(group_width),
            height: px(CHIP_SIZE),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    let Some(image) = game_assets.poker_chips.get(&denomination) else {
        return;
    };
    for index in 0..count {
        let chip = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(index as f32 * CHIP_REVEAL),
                    top: px(0),
                    width: px(CHIP_SIZE),
                    height: px(CHIP_SIZE),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ImageNode::new(image.clone()),
                ZIndex(index as i32),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(group).add_child(chip);
        let label_color = if denomination == 1 { HEADER_BG } else { TEXT };
        let label = add_text(
            commands,
            chip,
            denomination.to_string(),
            9.0,
            label_color,
            assets,
        );
        commands.entity(label).insert(TextShadow {
            offset: Vec2::new(0.7, 0.8),
            color: Color::BLACK.with_alpha(0.55),
        });
    }
}
