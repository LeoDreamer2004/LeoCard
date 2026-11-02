use super::{TexasChipTableState, TexasSpectatorUiState, TexasTableLayout, add_texas_win_rate};
use crate::app::presentation::{
    PlayerMenuProfile, PlayerPortraitSpec, SeatSide, TEXT, TurnBorderAnimationKey,
    TurnBorderMaterial, add_horizontal_chip_group, add_player_portrait, add_player_role_badge,
    add_player_role_badge_row, add_poker_chip_value, add_tabletop_balance_panel,
    add_turn_border_trace_with_radius, attach_start_game_seat_transition, card_background_image,
    position_opponent_popup,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::OpponentBadge;
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::{GameKind, TexasPlayerView, TexasSnapshot};
use leocard_ui::{UiComposer, spawn_node};

pub(super) const TEXAS_PORTRAIT_WIDTH: f32 = 96.0 * 1.17;
pub(super) const TEXAS_PORTRAIT_HEIGHT: f32 = 76.0 * 1.17 + 34.0;

#[expect(
    clippy::too_many_arguments,
    reason = "the opponent builder keeps seat state and visual resources explicit"
)]
pub(super) fn add_texas_opponent(
    commands: &mut UiComposer,
    table: Entity,
    player: &TexasPlayerView,
    relative: u8,
    game: &TexasSnapshot,
    interaction_menu_open: Option<PlayerId>,
    assets: &UiAssets,
    avatars: &AvatarImages,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    chip_state: &TexasChipTableState,
    start_transition_active: bool,
    intro_only: bool,
    spectator: &TexasSpectatorUiState,
) {
    let layout = TexasTableLayout::for_game(game);
    let side = layout.side(relative);
    let position = layout.portrait_position(relative);
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
    attach_start_game_seat_transition(commands, seat, player.player.id, start_transition_active);
    let portrait = add_player_portrait(
        commands,
        seat,
        Node {
            width: px(TEXAS_PORTRAIT_WIDTH),
            height: px(TEXAS_PORTRAIT_HEIGHT),
            ..default()
        },
        PlayerPortraitSpec {
            player: player.player.id,
            profile: PlayerMenuProfile {
                name: &player.player.name,
                avatar: player.player.avatar.and_then(|id| avatars.remote.get(&id)),
                reference_points: player.player.reference_points,
                completed_games: player.player.completed_games,
                game_profiles: &player.player.game_profiles,
            },
            avatar_size: 52.0 * 1.17,
            auto_play: player.auto_play,
            menu_open: interaction_menu_open == Some(player.player.id),
            name_color: TEXT,
        },
        assets,
    );
    if !player.folded && game.current_player == Some(player.player.id) {
        add_turn_border_trace_with_radius(
            commands,
            portrait.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Texas, game.match_id, player.player.id),
            52.0 * 1.17 * 0.2,
            52.0 * 1.17,
        );
    }
    add_role_tokens(
        commands,
        portrait.avatar_ring,
        player.player.id,
        game,
        assets,
    );
    if !intro_only {
        add_texas_win_rate(
            commands,
            portrait.avatar_ring,
            player.player.id,
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
        &chip_state.stack_counts(player.player.id),
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
    game: &TexasSnapshot,
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
    let row = add_player_role_badge_row(commands, avatar);
    for label in labels {
        add_player_role_badge(commands, row, label, assets);
    }
}

/// 账本中的实际筹码；自己的面板使用木质筹码框，对手悬停面板保持原样。
pub(super) fn add_texas_chip_popup(
    commands: &mut UiComposer,
    parent: Entity,
    stack: u32,
    opponent_side: Option<SeatSide>,
    assets: &UiAssets,
    stack_counts: &[(u16, usize)],
) -> Entity {
    let Some(side) = opponent_side else {
        return add_tabletop_balance_panel(
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
