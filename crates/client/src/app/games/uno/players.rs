use super::*;
use leocard_protocol::{
    GameKind, PlayerId, UnoPendingSwapView, UnoPhaseView, UnoPlayerState, UnoSnapshot,
};
use leocard_uno::UnoCard;

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
        &[(550.0, 28.0)],
        &[(275.0, 42.0), (825.0, 42.0)],
        &[(135.0, 112.0), (550.0, 24.0), (965.0, 112.0)],
        &[(85.0, 185.0), (330.0, 34.0), (770.0, 34.0), (1015.0, 185.0)],
        &[
            (65.0, 220.0),
            (225.0, 55.0),
            (550.0, 22.0),
            (875.0, 55.0),
            (1035.0, 220.0),
        ],
    ];
    POSITIONS[count.saturating_sub(1).min(4)][index]
}

#[allow(clippy::too_many_arguments)]
pub(super) fn add_uno_player_panel(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    player: &UnoPlayerState,
    (left, top): (f32, f32),
    ui: &UiState,
    avatars: &AvatarImages,
    assets: &UiAssets,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
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
    let selected = selecting && ui.uno.swap_targets.contains(&player.id);
    let panel = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: px(top),
            width: px(180),
            height: px(82),
            padding: UiRect::all(px(8)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(8),
            ..default()
        },
        if selected {
            Color::BLACK.with_alpha(0.76)
        } else if player.eliminated {
            Color::BLACK.with_alpha(0.78)
        } else if selecting {
            ACCENT.mix(&PANEL, 0.48).with_alpha(0.97)
        } else {
            PANEL.with_alpha(if player.connected { 0.92 } else { 0.58 })
        },
        PanelSkin::Section,
        assets,
    );
    if selecting {
        commands.entity(panel).insert((
            Button,
            UiAction::ToggleUnoSwapTarget(player.id),
            UnoSwapTargetPanel { selected },
            Outline::new(px(2.0), px(1.0), ACCENT.with_alpha(0.92)),
            BoxShadow::new(ACCENT.with_alpha(0.32), px(0), px(0), px(2), px(9)),
        ));
    } else {
        commands
            .entity(panel)
            .insert((Button, UiAction::ToggleInteractionMenu(player.id)));
    }
    if !player.eliminated
        && game.current_player == Some(player.id)
        && matches!(game.phase, UnoPhaseView::Playing)
    {
        add_turn_border_trace(
            commands,
            panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Uno, game.match_id, player.id),
        );
    }
    let avatar_handle = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar = add_avatar(commands, panel, &player.name, avatar_handle, 43.0, assets);
    commands
        .entity(avatar)
        .insert(PlayerAvatarAnchor(player.id));
    if player.id == game.host {
        add_host_crown(commands, avatar, assets);
    }
    let copy = spawn_node(
        commands,
        panel,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    add_text(commands, copy, &player.name, 14.0, TEXT, assets);
    add_text(
        commands,
        copy,
        format!(
            "{}{}",
            player.hand_len,
            if player.auto_play {
                " 张牌 · 托管"
            } else {
                " 张牌"
            }
        ),
        12.0,
        if game.uno_exposed.contains(&player.id) {
            DANGER
        } else {
            MUTED
        },
        assets,
    );
    let side = if left < DESIGN_WIDTH * 0.34 {
        SeatSide::Left
    } else if left > DESIGN_WIDTH * 0.66 {
        SeatSide::Right
    } else {
        SeatSide::Top
    };
    let menu = add_interaction_menu(
        commands,
        panel,
        player.id,
        side,
        PlayerMenuProfile {
            name: &player.name,
            avatar: avatar_handle,
            reference_points: player.reference_points,
            completed_games: player.completed_games,
            game_profiles: &player.game_profiles,
        },
        assets,
    );
    commands.entity(menu).insert(
        if !selecting && ui.social.interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    );
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: None,
        interaction_menu: menu,
    });
    if !player.inactive_hand.is_empty() {
        let row = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                left: px(8),
                top: px(86),
                height: px(54),
                align_items: AlignItems::FlexStart,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        let reveal = (156.0 / player.inactive_hand.len().max(1) as f32).clamp(8.0, 20.0);
        for (index, card) in player.inactive_hand.iter().copied().enumerate() {
            let slot = spawn_node(
                commands,
                row,
                Node {
                    width: px(if index + 1 == player.inactive_hand.len() {
                        34.0
                    } else {
                        reveal
                    }),
                    height: px(52),
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
                        width: px(34),
                        height: px(52),
                        ..default()
                    },
                    ImageNode::new(uno_card_handle(assets, card)),
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
        add_uno_skip_overlay(commands, panel, uno_skip_count(game, player), assets);
    }
    if let Some(cards) = uno_finished_hand(game, player.id)
        && !cards.is_empty()
    {
        add_uno_finished_hand(commands, panel, cards, assets);
    }
    if player.eliminated {
        add_uno_eliminated_player_overlay(commands, panel, assets);
    }
}

fn add_uno_eliminated_player_overlay(commands: &mut Commands, panel: Entity, assets: &UiAssets) {
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
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.58)),
    );
    commands.entity(overlay).insert((
        BorderColor::all(MUTED.with_alpha(0.42)),
        ZIndex(18),
        FocusPolicy::Pass,
    ));
    let badge = spawn_node(
        commands,
        overlay,
        Node {
            padding: UiRect::axes(px(12), px(5)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.72)),
    );
    commands.entity(badge).insert((
        BorderColor::all(TEXT.with_alpha(0.34)),
        BoxShadow::new(Color::BLACK.with_alpha(0.42), px(1), px(3), px(0), px(5)),
        FocusPolicy::Pass,
    ));
    let label = add_text(
        commands,
        badge,
        "已淘汰",
        14.0,
        TEXT.with_alpha(0.86),
        assets,
    );
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
            right: px(7),
            top: px(7),
            padding: UiRect::axes(px(7), px(3)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(ACCENT.with_alpha(0.84)),
    );
    commands
        .entity(badge)
        .insert((GlobalZIndex(8), FocusPolicy::Pass));
    add_text(commands, badge, "已选中", 11.0, Color::BLACK, assets);
}

pub(super) fn add_uno_swap_selection_prompt(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    ui: &UiState,
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
    let ready = required.is_some_and(|count| ui.uno.swap_targets.len() == count);
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
    let panel = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(475),
            top: px(118),
            width: px(330),
            padding: UiRect::axes(px(14), px(10)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(8),
            ..default()
        },
        PANEL.with_alpha(0.96),
        PanelSkin::Popup,
        assets,
    );
    commands.entity(panel).insert(GlobalZIndex(45));
    add_text(commands, panel, title, 14.0, TEXT, assets);
    if required.is_some() {
        if ready {
            add_action_button(
                commands,
                panel,
                "确定选择",
                UiAction::ConfirmUnoSwapTargets,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, panel, "确定选择", assets);
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
    assets: &UiAssets,
) {
    const WIDTH: f32 = 172.0;
    const CARD_WIDTH: f32 = 44.0;
    const CARD_HEIGHT: f32 = 68.0;
    let reveal = if cards.len() <= 1 {
        CARD_WIDTH
    } else {
        ((WIDTH - CARD_WIDTH) / cards.len().saturating_sub(1) as f32).clamp(2.5, 22.0)
    };
    let hand = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(4),
            top: px(86),
            width: px(WIDTH),
            height: px(CARD_HEIGHT + 6.0),
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
                    CARD_WIDTH
                } else {
                    reveal
                }),
                height: px(CARD_HEIGHT),
                flex_shrink: 0.0,
                overflow: Overflow::visible(),
                ..default()
            },
            None,
        );
        let face = commands
            .spawn((
                Node {
                    width: px(CARD_WIDTH),
                    height: px(CARD_HEIGHT),
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
            border_radius: BorderRadius::all(px(8)),
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
            width: px(58),
            height: px(58),
            position_type: PositionType::Relative,
            border: UiRect::all(px(5)),
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
            left: px(-4),
            top: px(20),
            width: px(66),
            height: px(9),
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
                right: px(7),
                bottom: px(5),
                min_width: px(37),
                height: px(25),
                padding: UiRect::horizontal(px(7)),
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
        add_text(commands, badge, count.to_string(), 15.0, TEXT, assets);
    }
}
