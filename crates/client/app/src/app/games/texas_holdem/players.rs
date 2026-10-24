use super::{
    TexasChipTableState, TexasPlayerPanel, TexasPlayerShake, TexasSpectatorUiState,
    add_texas_win_rate, texas_player_border_color, texas_player_chip_zone,
};
use crate::app::presentation::{
    ACCENT, HEADER_BG, MUTED, PlayerMenuProfile, PlayerPortraitSpec, TEXT, TurnBorderAnimationKey,
    TurnBorderMaterial, UiComposer, add_horizontal_chip_group, add_player_portrait,
    add_poker_balance_panel, add_poker_chip_value, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, card_background_image, position_opponent_popup, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{OpponentBadge, SeatSide};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_protocol::{GameKind, PlayerId, TexasHoldemPlayerState, TexasHoldemSnapshot};

pub(super) const TEXAS_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(super) const TEXAS_PORTRAIT_HEIGHT: f32 = 76.0 * 1.17 + 34.0;

pub(super) fn texas_portrait_position(relative: u8) -> Vec2 {
    let side = match relative {
        0..=2 => SeatSide::Left,
        3 => SeatSide::Top,
        _ => SeatSide::Right,
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
    Vec2::new(left, top)
}

#[expect(
    clippy::too_many_arguments,
    reason = "the opponent builder keeps seat state and visual resources explicit"
)]
pub(super) fn add_texas_opponent(
    commands: &mut UiComposer,
    table: Entity,
    player: &TexasHoldemPlayerState,
    relative: u8,
    game: &TexasHoldemSnapshot,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
    intro_only: bool,
    spectator: &TexasSpectatorUiState,
) {
    let side = match relative {
        0..=2 => SeatSide::Left,
        3 => SeatSide::Top,
        4 | 5 => SeatSide::Right,
        _ => return,
    };
    let position = texas_portrait_position(relative);
    let (left, top) = (position.x, position.y);
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
            avatar_size: 52.0 * 1.17,
            auto_play: player.auto_play,
            menu_open: interaction_menu_open == Some(player.id),
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
    if !intro_only {
        add_texas_win_rate(
            commands,
            portrait.avatar_ring,
            player.id,
            side,
            game,
            spectator,
            assets,
        );
    }
    add_texas_stack_value(commands, portrait.portrait, player.stack, assets);
    if intro_only {
        return;
    }

    let popup = add_texas_chip_popup(
        commands,
        seat,
        player.stack,
        Some(side),
        assets,
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
    commands: &mut UiComposer,
    portrait: Entity,
    stack: u32,
    assets: &UiAssets,
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
    commands.entity(area).insert(Pickable::IGNORE);
    add_poker_chip_value(commands, area, stack, assets);
}

pub(super) fn add_role_tokens(
    commands: &mut UiComposer,
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
    commands.entity(row).insert((ZIndex(45), Pickable::IGNORE));
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
        commands.entity(chip).insert(Pickable::IGNORE);
        add_text(commands, chip, label, 8.0, HEADER_BG, assets);
    }
}

/// 自己与其他玩家的筹码面板共用首页游戏卡片纹理，显示账本中的实际筹码。
pub(super) fn add_texas_chip_popup(
    commands: &mut UiComposer,
    parent: Entity,
    stack: u32,
    opponent_side: Option<SeatSide>,
    assets: &UiAssets,
    stack_counts: &[(u16, usize)],
) -> Entity {
    let Some(side) = opponent_side else {
        return add_poker_balance_panel(
            commands,
            parent,
            Vec2::new(TEXAS_PORTRAIT_WIDTH, 76.0 * 1.17),
            stack,
            stack_counts,
            assets,
        );
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        width: px(330),
        padding: UiRect::all(px(6)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Stretch,
        row_gap: px(3),
        ..default()
    };
    position_opponent_popup(&mut node, side);
    node.top = px(TEXAS_PORTRAIT_HEIGHT + 6.0);
    let popup = spawn_node(commands, parent, node, None);
    commands
        .entity(popup)
        .insert((GlobalZIndex(1500), Pickable::IGNORE));
    let image = card_background_image(assets.home.game_card.clone(), 1.0);
    commands.entity(popup).insert(image);

    let chips = spawn_node(
        commands,
        popup,
        Node {
            width: percent(100),
            height: px(38),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            column_gap: px(18),
            ..default()
        },
        None,
    );
    for &(denomination, count) in stack_counts {
        add_horizontal_chip_group(commands, chips, denomination, count, assets);
    }
    popup
}
