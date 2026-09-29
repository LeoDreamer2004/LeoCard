use super::{
    UnoAssets, UnoFlipTarget, UnoSwapTargetPanel, UnoUiAction, UnoUiState, add_uno_action_button,
    add_uno_disabled_action_button, uno_card_handle,
};
use crate::app::presentation::{
    ACCENT, ButtonKind, DANGER, DESIGN_WIDTH, MUTED, PanelSkin, PlayerMenuProfile,
    PlayerPortraitSpec, TEXT, TurnBorderAnimationKey, TurnBorderMaterial, add_host_crown,
    add_player_portrait, add_text, add_turn_border_trace_with_radius,
    attach_start_game_seat_transition, spawn_node,
};
use crate::app::runtime::{AvatarImages, UiAssets};
use crate::app::shell::{SeatSide, SocialUiState, UiAction, add_cozy_panel_with_skin};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{
    GameKind, PlayerId, UnoPendingSwapView, UnoPhaseView, UnoPlayerState, UnoSnapshot,
};
use leocard_uno::UnoCard;

const UNO_OPPONENT_HAND_WIDTH: f32 = 172.0;
const UNO_OPPONENT_CARD_WIDTH: f32 = 44.0;
const UNO_OPPONENT_CARD_HEIGHT: f32 = 68.0;
pub(super) const UNO_PORTRAIT_WIDTH: f32 = 104.0 * 1.17;
pub(super) const UNO_PORTRAIT_HEIGHT: f32 = 106.0 * 1.17;
pub(super) const UNO_AVATAR_SIZE: f32 = 52.0 * 1.17;

pub(super) fn add_uno_eliminated_own_overlay(
    commands: &mut Commands,
    parent: Entity,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            bottom: px(0),
            height: px(190),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(5),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.82)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(1900), FocusPolicy::Block));
    let title = add_text(commands, overlay, "您已被淘汰", 27.0, DANGER, assets);
    commands.entity(title).insert((
        FocusPolicy::Pass,
        TextShadow {
            offset: Vec2::new(1.2, 1.5),
            color: Color::BLACK.with_alpha(0.92),
        },
    ));
    let detail = add_text(commands, overlay, "等待本局结束", 14.0, MUTED, assets);
    commands.entity(detail).insert(FocusPolicy::Pass);
}

pub(super) fn opponent_position(index: usize, count: usize) -> (f32, f32) {
    const POSITIONS: [&[(f32, f32)]; 5] = [
        &[(550.0, 10.0)],
        &[(235.0, 52.0), (865.0, 52.0)],
        &[(100.0, 140.0), (550.0, 8.0), (1000.0, 140.0)],
        &[(60.0, 215.0), (300.0, 16.0), (800.0, 16.0), (1040.0, 215.0)],
        &[
            (40.0, 252.0),
            (205.0, 38.0),
            (550.0, 8.0),
            (895.0, 38.0),
            (1060.0, 252.0),
        ],
    ];
    let (left, top) = POSITIONS[count.saturating_sub(1).min(4)][index];
    (left, top + 24.0)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn add_uno_player_panel(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    player: &UnoPlayerState,
    (left, top): (f32, f32),
    ui: &UnoUiState,
    social: &SocialUiState,
    avatars: &AvatarImages,
    assets: &UiAssets,
    game_assets: &UnoAssets,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    start_transition_active: bool,
    intro_only: bool,
) {
    let selecting = match game.pending_swap {
        Some(UnoPendingSwapView::SwapOneTarget { player: actor }) => {
            actor == game.you && player.id != game.you && !player.eliminated
        }
        Some(UnoPendingSwapView::ForceTrade { player: actor }) => {
            actor == game.you && !player.eliminated
        }
        Some(UnoPendingSwapView::SevenSwap { player: actor }) => {
            actor == game.you && player.id != game.you && !player.eliminated
        }
        _ => false,
    };
    let selected = selecting && ui.swap_targets.contains(&player.id);
    let side = if left < DESIGN_WIDTH * 0.34 {
        SeatSide::Left
    } else if left > DESIGN_WIDTH * 0.66 {
        SeatSide::Right
    } else {
        SeatSide::Top
    };
    let avatar_handle = player.avatar.and_then(|id| avatars.remote.get(&id));
    let portrait = add_player_portrait(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left + (180.0 - UNO_PORTRAIT_WIDTH) * 0.5),
            top: px(top),
            width: px(UNO_PORTRAIT_WIDTH),
            height: px(UNO_PORTRAIT_HEIGHT),
            ..default()
        },
        PlayerPortraitSpec {
            player: player.id,
            profile: PlayerMenuProfile {
                name: &player.name,
                avatar: avatar_handle,
                reference_points: player.reference_points,
                completed_games: player.completed_games,
                game_profiles: &player.game_profiles,
            },
            side,
            avatar_size: UNO_AVATAR_SIZE,
            auto_play: player.auto_play,
            menu_open: !selecting && social.interaction_menu_open == Some(player.id),
            menu_above: false,
            name_color: if player.connected { TEXT } else { MUTED },
        },
        assets,
    );
    let panel = portrait.portrait;
    attach_start_game_seat_transition(commands, panel, player.id, start_transition_active);
    if intro_only {
        return;
    }
    commands
        .entity(portrait.avatar_ring)
        .entry::<Node>()
        .and_modify(|mut node| {
            node.border_radius = BorderRadius::all(px(UNO_AVATAR_SIZE * 0.2));
        });
    if selecting {
        commands
            .entity(panel)
            .insert(UiAction::Uno(UnoUiAction::ToggleSwapTarget(player.id)));
        commands.entity(portrait.avatar_ring).insert((
            UnoSwapTargetPanel { selected },
            BackgroundColor(Color::NONE),
            Outline::new(px(2.0), px(1.0), ACCENT.with_alpha(0.92)),
            BoxShadow::new(ACCENT.with_alpha(0.32), px(0), px(0), px(2), px(9)),
        ));
    }
    if !player.eliminated
        && game.current_player == Some(player.id)
        && matches!(game.phase, UnoPhaseView::Playing)
    {
        add_turn_border_trace_with_radius(
            commands,
            portrait.avatar_ring,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Uno, game.match_id, player.id),
            UNO_AVATAR_SIZE * 0.2,
            UNO_AVATAR_SIZE,
        );
    }
    if player.id == game.host {
        add_host_crown(commands, portrait.avatar_ring, assets);
    }
    add_uno_card_count(
        commands,
        panel,
        player.hand_len.into(),
        if game.uno_exposed.contains(&player.id) {
            DANGER
        } else {
            MUTED
        },
        assets,
    );
    let finished_hand = uno_finished_hand(game, player.id);
    if finished_hand.is_none() && !player.inactive_hand.is_empty() {
        let row = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                left: px((UNO_PORTRAIT_WIDTH - UNO_OPPONENT_HAND_WIDTH) * 0.5),
                top: px(UNO_PORTRAIT_HEIGHT + 4.0),
                width: px(UNO_OPPONENT_HAND_WIDTH),
                height: px(UNO_OPPONENT_CARD_HEIGHT + 6.0),
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        let reveal = uno_opponent_hand_reveal(player.inactive_hand.len());
        for (index, card) in player.inactive_hand.iter().copied().enumerate() {
            let slot = spawn_node(
                commands,
                row,
                Node {
                    width: px(if index + 1 == player.inactive_hand.len() {
                        UNO_OPPONENT_CARD_WIDTH
                    } else {
                        reveal
                    }),
                    height: px(UNO_OPPONENT_CARD_HEIGHT),
                    flex_shrink: 0.0,
                    overflow: Overflow::visible(),
                    ..default()
                },
                None,
            );
            let face = commands
                .spawn((
                    UnoFlipTarget::Opponent {
                        player: player.id,
                        index,
                    },
                    Node {
                        width: px(UNO_OPPONENT_CARD_WIDTH),
                        height: px(UNO_OPPONENT_CARD_HEIGHT),
                        ..default()
                    },
                    ImageNode::new(uno_card_handle(game_assets, card)),
                    UiTransform::IDENTITY,
                    BoxShadow::new(Color::BLACK.with_alpha(0.35), px(1), px(2), px(0), px(3)),
                    FocusPolicy::Pass,
                ))
                .id();
            commands.entity(slot).add_child(face);
        }
    }
    if selected {
        add_uno_swap_selected_label(commands, panel, assets);
    }
    if !player.eliminated {
        add_uno_skip_overlay(
            commands,
            portrait.avatar_ring,
            uno_skip_count(game, player),
            assets,
        );
    }
    if let Some(cards) = finished_hand
        && !cards.is_empty()
    {
        add_uno_finished_hand(commands, panel, cards, game_assets);
    }
    if player.eliminated {
        add_uno_eliminated_player_overlay(commands, portrait.avatar_ring, assets);
    }
}

/// 三张散开的牌背与剩余手牌数，放在头像姓名正下方。
pub(super) fn add_uno_card_count(
    commands: &mut Commands,
    portrait: Entity,
    count: usize,
    color: Color,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        portrait,
        Node {
            position_type: PositionType::Absolute,
            top: px(78.0 * 1.17),
            left: px(0),
            width: percent(100),
            height: px(27.0 * 1.17),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(5.0 * 1.17),
            ..default()
        },
        None,
    );
    commands.entity(row).insert(FocusPolicy::Pass);
    let fan = spawn_node(
        commands,
        row,
        Node {
            width: px(34.0 * 1.17),
            height: px(27.0 * 1.17),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    commands.entity(fan).insert(FocusPolicy::Pass);
    for (left, top, rotation, layer) in [
        (2.0, 5.0, -0.27, 0),
        (10.0, 2.0, 0.0, 1),
        (18.0, 5.0, 0.27, 2),
    ] {
        let back = spawn_node(
            commands,
            fan,
            Node {
                position_type: PositionType::Absolute,
                left: px(left * 1.17),
                top: px(top * 1.17),
                width: px(15.0 * 1.17),
                height: px(22.0 * 1.17),
                border: UiRect::all(px(1.17)),
                border_radius: BorderRadius::all(px(2.0 * 1.17)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(Color::srgb(0.035, 0.045, 0.055)),
        );
        commands.entity(back).insert((
            BorderColor::all(Color::srgb(0.93, 0.94, 0.92)),
            UiTransform::from_rotation(Rot2::radians(rotation)),
            ZIndex(layer),
            FocusPolicy::Pass,
        ));
        let diamond = spawn_node(
            commands,
            back,
            Node {
                width: px(7.0 * 1.17),
                height: px(7.0 * 1.17),
                ..default()
            },
            Some(Color::srgb(0.92, 0.08, 0.10)),
        );
        commands.entity(diamond).insert((
            UiTransform::from_rotation(Rot2::degrees(45.0)),
            FocusPolicy::Pass,
        ));
    }
    let number = add_text(commands, row, count.to_string(), 14.0 * 1.17, color, assets);
    commands.entity(number).insert((
        TextLayout::default().with_no_wrap(),
        TextShadow {
            offset: Vec2::new(1.0, 1.0),
            color: Color::BLACK.with_alpha(0.8),
        },
        FocusPolicy::Pass,
    ));
}

pub(super) fn add_uno_eliminated_player_overlay(
    commands: &mut Commands,
    panel: Entity,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(UNO_AVATAR_SIZE * 0.2)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.92)),
    );
    commands.entity(overlay).insert((
        BorderColor::all(DANGER.with_alpha(0.96)),
        ZIndex(80),
        FocusPolicy::Pass,
    ));
    let label = add_text(commands, overlay, "OUT", 19.0, DANGER, assets);
    commands.entity(label).insert(FocusPolicy::Pass);
}

pub(super) fn add_uno_swap_selected_label(
    commands: &mut Commands,
    panel: Entity,
    assets: &UiAssets,
) {
    let badge = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            right: px(7.0 * 1.17),
            top: px(7.0 * 1.17),
            padding: UiRect::axes(px(7.0 * 1.17), px(3.0 * 1.17)),
            border_radius: BorderRadius::all(px(8.0 * 1.17)),
            ..default()
        },
        Some(ACCENT.with_alpha(0.84)),
    );
    commands
        .entity(badge)
        .insert((GlobalZIndex(8), FocusPolicy::Pass));
    add_text(commands, badge, "已选中", 11.0 * 1.17, Color::BLACK, assets);
}

pub(super) fn add_uno_swap_selection_prompt(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    ui: &UnoUiState,
    assets: &UiAssets,
) {
    let Some(pending) = game.pending_swap else {
        return;
    };
    let (actor, title, required) = match pending {
        UnoPendingSwapView::SwapOneTarget { player } => (player, "选择一名玩家交换一张牌", Some(1)),
        UnoPendingSwapView::SwapOneGive { player, target } => {
            let target_name = game
                .players
                .iter()
                .find(|candidate| candidate.id == target)
                .map(|candidate| candidate.name.as_str())
                .unwrap_or("目标玩家");
            let title = if player == game.you {
                format!("已从 {target_name} 随机取得一张牌，请选择一张交还")
            } else {
                format!("等待玩家向 {target_name} 交还一张牌")
            };
            add_uno_swap_prompt_panel(commands, table, &title, None, false, assets);
            return;
        }
        UnoPendingSwapView::ForceTrade { player } => (player, "选择两名玩家交换整手牌", Some(2)),
        UnoPendingSwapView::SevenSwap { player } => (player, "选择一名玩家交换整手牌", Some(1)),
        UnoPendingSwapView::ChooseColor { .. } | UnoPendingSwapView::ColorRoulette { .. } => return,
    };
    let own_turn = actor == game.you;
    let actor_name = game
        .players
        .iter()
        .find(|candidate| candidate.id == actor)
        .map(|candidate| candidate.name.as_str())
        .unwrap_or("玩家");
    let title = if own_turn {
        title.to_owned()
    } else {
        format!("等待 {actor_name} 选择换牌目标")
    };
    let ready = required.is_some_and(|count| ui.swap_targets.len() == count);
    add_uno_swap_prompt_panel(commands, table, &title, required, own_turn && ready, assets);
}

fn add_uno_swap_prompt_panel(
    commands: &mut Commands,
    table: Entity,
    title: &str,
    required: Option<usize>,
    ready: bool,
    assets: &UiAssets,
) {
    let panel = add_cozy_panel_with_skin(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(475),
            top: px(118),
            width: px(330),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(8),
            ..default()
        },
        PanelSkin::Popup,
        assets,
    );
    commands.entity(panel).insert(GlobalZIndex(45));
    add_text(commands, panel, title, 14.0, TEXT, assets);
    if required.is_some() {
        if ready {
            add_uno_action_button(
                commands,
                panel,
                "确定选择",
                UiAction::Uno(UnoUiAction::ConfirmSwapTargets),
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_uno_disabled_action_button(commands, panel, "确定选择", assets);
        }
    }
}

fn uno_finished_hand(game: &UnoSnapshot, player: PlayerId) -> Option<&[UnoCard]> {
    let UnoPhaseView::Finished {
        remaining_hands, ..
    } = &game.phase
    else {
        return None;
    };
    remaining_hands
        .iter()
        .find(|hand| hand.player == player)
        .map(|hand| hand.cards.as_slice())
}

/// 终局停顿期间把对手的实际手牌摊开在人像框下方。牌再多也压缩间距，
/// 保证整手牌仍归属于对应玩家，而不会挤入相邻座位。
fn add_uno_finished_hand(
    commands: &mut Commands,
    panel: Entity,
    cards: &[UnoCard],
    assets: &UnoAssets,
) {
    let reveal = uno_opponent_hand_reveal(cards.len());
    let hand = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px((UNO_PORTRAIT_WIDTH - UNO_OPPONENT_HAND_WIDTH) * 0.5),
            top: px(UNO_PORTRAIT_HEIGHT + 4.0),
            width: px(UNO_OPPONENT_HAND_WIDTH),
            height: px(UNO_OPPONENT_CARD_HEIGHT + 6.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );
    commands
        .entity(hand)
        .insert((GlobalZIndex(850), FocusPolicy::Pass));
    for (index, card) in cards.iter().copied().enumerate() {
        let slot = spawn_node(
            commands,
            hand,
            Node {
                width: px(if index + 1 == cards.len() {
                    UNO_OPPONENT_CARD_WIDTH
                } else {
                    reveal
                }),
                height: px(UNO_OPPONENT_CARD_HEIGHT),
                flex_shrink: 0.0,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        let face = commands
            .spawn((
                Node {
                    width: px(UNO_OPPONENT_CARD_WIDTH),
                    height: px(UNO_OPPONENT_CARD_HEIGHT),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                ImageNode::new(uno_card_handle(assets, card)),
                BoxShadow::new(Color::BLACK.with_alpha(0.42), px(1), px(3), px(0), px(4)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(slot).add_child(face);
    }
}

fn uno_opponent_hand_reveal(card_count: usize) -> f32 {
    if card_count <= 1 {
        UNO_OPPONENT_CARD_WIDTH
    } else {
        ((UNO_OPPONENT_HAND_WIDTH - UNO_OPPONENT_CARD_WIDTH) / card_count.saturating_sub(1) as f32)
            .clamp(2.5, 22.0)
    }
}

pub(super) fn uno_skip_count(game: &UnoSnapshot, player: &UnoPlayerState) -> u16 {
    player.skipped_turns
        + if game.current_player == Some(player.id) {
            game.pending_skip
        } else {
            0
        }
}

pub(super) fn add_uno_skip_overlay(
    commands: &mut Commands,
    panel: Entity,
    count: u16,
    assets: &UiAssets,
) {
    if count == 0 {
        return;
    }
    let overlay = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(UNO_AVATAR_SIZE * 0.2)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.72)),
    );
    commands.entity(overlay).insert((
        BorderColor::all(DANGER.with_alpha(0.96)),
        FocusPolicy::Pass,
        ZIndex(80),
    ));
    let symbol = spawn_node(
        commands,
        overlay,
        Node {
            width: px(39.0 * 1.17),
            height: px(39.0 * 1.17),
            position_type: PositionType::Relative,
            border: UiRect::all(px(4.0 * 1.17)),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        None,
    );
    commands.entity(symbol).insert((
        BorderColor::all(DANGER),
        BoxShadow::new(Color::BLACK.with_alpha(0.58), px(2), px(3), px(0), px(5)),
        FocusPolicy::Pass,
    ));
    let slash = spawn_node(
        commands,
        symbol,
        Node {
            position_type: PositionType::Absolute,
            left: px(-3.0 * 1.17),
            top: px(13.0 * 1.17),
            width: px(43.0 * 1.17),
            height: px(7.0 * 1.17),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(DANGER),
    );
    commands.entity(slash).insert((
        UiTransform::from_rotation(Rot2::degrees(-45.0)),
        BoxShadow::new(Color::BLACK.with_alpha(0.60), px(2), px(3), px(0), px(4)),
        FocusPolicy::Pass,
    ));
    if count > 1 {
        let badge = spawn_node(
            commands,
            overlay,
            Node {
                position_type: PositionType::Absolute,
                right: px(-7.0 * 1.17),
                bottom: px(-5.0 * 1.17),
                min_width: px(25.0 * 1.17),
                height: px(20.0 * 1.17),
                padding: UiRect::horizontal(px(5.0 * 1.17)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            Some(Color::BLACK.with_alpha(0.82)),
        );
        commands.entity(badge).insert((
            BorderColor::all(DANGER),
            BoxShadow::new(Color::BLACK.with_alpha(0.5), px(1), px(2), px(0), px(3)),
            FocusPolicy::Pass,
        ));
        add_text(
            commands,
            badge,
            count.to_string(),
            12.0 * 1.17,
            TEXT,
            assets,
        );
    }
}
