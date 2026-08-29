//! UNO 牌桌视图。

use super::*;

const UNO_DISCARD_OFFSETS: [(f32, f32, f32); 6] = [
    (-5.0, 4.0, -5.0),
    (4.0, 2.0, 4.0),
    (-2.0, -2.0, -2.5),
    (3.0, 1.0, 3.0),
    (-1.0, 0.0, -1.5),
    (1.0, -1.0, 2.0),
];
const UNO_FLYING_CARD_WIDTH: f32 = 82.0;
const UNO_FLYING_CARD_HEIGHT: f32 = 128.0;
const UNO_PALETTE_EFFECT_DURATION: f32 = 2.2;
const UNO_REVERSE_EFFECT_DURATION: f32 = 1.65;
const UNO_ACTION_AREA_BOTTOM: f32 = 153.0;
const UNO_ACTION_AREA_HEIGHT: f32 = 52.0;
const UNO_PALETTE_SHADER: &str = "shaders/uno_palette.wgsl";

/// GPU 直接绘制调色轮。params: x=选中色编号，y=0 底盘/1 独立扇区，z=透明度。
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub(super) struct UnoPaletteMaterial {
    #[uniform(0)]
    params: Vec4,
}

impl UnoPaletteMaterial {
    fn new(selected: UnoColor, selected_sector: bool) -> Self {
        let selected = match selected {
            UnoColor::Red => 0.0,
            UnoColor::Yellow => 1.0,
            UnoColor::Green => 2.0,
            UnoColor::Blue => 3.0,
            UnoColor::Pink => 4.0,
            UnoColor::Teal => 5.0,
            UnoColor::Orange => 6.0,
            UnoColor::Purple => 7.0,
        };
        Self {
            params: Vec4::new(selected, if selected_sector { 1.0 } else { 0.0 }, 0.0, 0.0),
        }
    }
}

impl UiMaterial for UnoPaletteMaterial {
    fn fragment_shader() -> ShaderRef {
        UNO_PALETTE_SHADER.into()
    }
}

pub(in crate::app) struct UnoTableVisuals<'a> {
    pub(in crate::app) assets: &'a UiAssets,
    pub(in crate::app) avatars: &'a AvatarImages,
    pub(in crate::app) appearance: &'a TableAppearance,
    pub(in crate::app) brightness: f32,
    pub(in crate::app) vignette: f32,
    pub(in crate::app) table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub(in crate::app) turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub(in crate::app) game_summary: &'a GameSummaryAnimation,
}

pub(in crate::app) fn render_uno_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &UnoSnapshot,
    ui: &UiState,
    chat: &ChatPanelState,
    visuals: UnoTableVisuals<'_>,
) {
    let UnoTableVisuals {
        assets,
        avatars,
        appearance,
        brightness,
        vignette,
        table_materials,
        turn_border_materials,
        game_summary,
    } = visuals;
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let felt = appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&assets.table_felt)
        .clone();
    let material = table_materials.add(TableBackgroundMaterial {
        params: table_material_params(brightness, vignette, appearance.custom_felt.is_none()),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));
    if game.flip_side == Some(leocard_uno::FlipSide::Dark) {
        let tint = spawn_node(
            commands,
            content,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Some(Color::srgba(0.12, 0.07, 0.30, 0.24)),
        );
        commands.entity(tint).insert(FocusPolicy::Pass);
    }

    let table = spawn_node(
        commands,
        content,
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: px(0),
            bottom: px(0),
            width: px(DESIGN_WIDTH),
            min_height: px(430),
            ..default()
        },
        None,
    );
    commands
        .entity(table)
        .insert(UiTransform::from_translation(Val2::px(
            -DESIGN_WIDTH * 0.5,
            0.0,
        )));

    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, content, message, assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("UNO 快照必须包含接收方");
    let mut opponents = game
        .players
        .iter()
        .filter(|player| player.id != game.you)
        .collect::<Vec<_>>();
    opponents.sort_by_key(|player| {
        (player.seat.0 + UnoRuleSet::MAX_PLAYERS - own.seat.0) % UnoRuleSet::MAX_PLAYERS
    });
    let opponent_count = opponents.len();
    for (index, player) in opponents.into_iter().enumerate() {
        add_uno_player_panel(
            commands,
            table,
            game,
            player,
            opponent_position(index, opponent_count),
            ui,
            avatars,
            assets,
            turn_border_materials,
        );
    }

    add_uno_center(commands, table, game, assets);
    add_uno_own_area(
        commands,
        table,
        game,
        own,
        ui,
        assets,
        turn_border_materials,
    );
    add_uno_actions(commands, table, game, ui, assets);
    add_uno_callout_actions(commands, table, game, assets);
    add_uno_swap_selection_prompt(commands, table, game, ui, assets);

    let needs_color_choice = game.current_color.is_none()
        && matches!(game.phase, UnoPhaseView::Playing)
        && (game.pending_swap.is_none()
            || matches!(
                game.pending_swap,
                Some(
                    UnoPendingSwapView::ChooseColor { .. }
                        | UnoPendingSwapView::ColorRoulette { .. }
                )
            ));
    if needs_color_choice {
        add_initial_color_choice(commands, content, game, assets);
    }
    if let Some(card) = ui.uno_color_choice {
        add_play_color_choice(commands, content, game.flip_side, card, assets);
    }
    if let UnoPhaseView::Finished {
        winner,
        results,
        reference_changes,
        ..
    } = &game.phase
    {
        add_uno_summary(
            commands,
            table,
            game,
            *winner,
            results,
            reference_changes,
            assets,
            avatars,
            game_summary,
        );
    }

    let auto_play = own.auto_play;
    add_chat_panel(commands, content, chat, assets, Some(auto_play), None, None);
    if own.eliminated && matches!(game.phase, UnoPhaseView::Playing) {
        add_uno_eliminated_own_overlay(commands, content, assets);
    } else if auto_play && matches!(game.phase, UnoPhaseView::Playing) {
        add_auto_play_overlay(commands, content, assets);
    }
}

fn add_uno_eliminated_own_overlay(commands: &mut Commands, parent: Entity, assets: &UiAssets) {
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

fn opponent_position(index: usize, count: usize) -> (f32, f32) {
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
fn add_uno_player_panel(
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
    let selected = selecting && ui.uno_swap_targets.contains(&player.id);
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
        &player.name,
        avatar_handle,
        player.reference_points,
        player.completed_games,
        &player.game_profiles,
        assets,
    );
    commands.entity(menu).insert(
        if !selecting && ui.interaction_menu_open == Some(player.id) {
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
                    Node {
                        width: px(34),
                        height: px(52),
                        ..default()
                    },
                    ImageNode::new(uno_card_handle(assets, card)),
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

fn add_uno_swap_selected_label(commands: &mut Commands, panel: Entity, assets: &UiAssets) {
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

fn add_uno_swap_selection_prompt(
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
    let ready = required.is_some_and(|count| ui.uno_swap_targets.len() == count);
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

fn uno_skip_count(game: &UnoSnapshot, player: &UnoPlayerState) -> u16 {
    player.skipped_turns
        + if game.current_player == Some(player.id) {
            game.pending_skip
        } else {
            0
        }
}

fn add_uno_skip_overlay(commands: &mut Commands, panel: Entity, count: u16, assets: &UiAssets) {
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

fn add_uno_center(commands: &mut Commands, table: Entity, game: &UnoSnapshot, assets: &UiAssets) {
    let center = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(467),
            top: px(205),
            width: px(346),
            height: px(170),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(34),
            ..default()
        },
        None,
    );
    let draw = commands
        .spawn((
            UnoDrawPileAnchor,
            Node {
                width: px(82),
                height: px(128),
                position_type: PositionType::Relative,
                ..default()
            },
            ImageNode::new(
                game.draw_pile_inactive_top
                    .map(|card| uno_card_handle(assets, card))
                    .unwrap_or_else(|| assets.uno_card_back.clone()),
            ),
            BoxShadow::new(Color::BLACK.with_alpha(0.45), px(3), px(5), px(0), px(7)),
        ))
        .id();
    commands.entity(center).add_child(draw);
    let draw_count = spawn_node(
        commands,
        draw,
        Node {
            position_type: PositionType::Absolute,
            right: px(-13),
            top: px(-10),
            min_width: px(31),
            height: px(25),
            padding: UiRect::horizontal(px(6)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    add_text(
        commands,
        draw_count,
        game.draw_pile_len.to_string(),
        11.0,
        TEXT,
        assets,
    );

    let discard = spawn_node(
        commands,
        center,
        Node {
            width: px(96),
            height: px(140),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    for (index, card) in game.discard_pile.iter().copied().enumerate() {
        let (x, y, angle) = uno_discard_pose(card);
        let mut card_entity = commands.spawn((
            UnoDiscardCard(card),
            Node {
                position_type: PositionType::Absolute,
                left: px(7.0 + x),
                top: px(6.0 + y),
                width: px(82),
                height: px(128),
                ..default()
            },
            ImageNode::new(uno_card_handle(assets, card)),
            UiTransform::from_rotation(Rot2::degrees(angle)),
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(4), px(0), px(5)),
        ));
        if index + 1 == game.discard_pile.len() {
            card_entity.insert(UnoDiscardPileAnchor);
        }
        let card = card_entity.id();
        commands.entity(discard).add_child(card);
    }

    let status = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(440),
            top: px(382),
            width: px(400),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let current_name = game
        .current_player
        .and_then(|id| game.players.iter().find(|player| player.id == id))
        .map(|player| player.name.as_str())
        .unwrap_or("—");
    let color = game.current_color.map_or("等待选色".to_owned(), |color| {
        format!("当前颜色：{color}")
    });
    add_text(
        commands,
        status,
        format!("{color}  ·  {current_name} 的回合"),
        16.0,
        game.current_color.map_or(ACCENT, uno_ui_color),
        assets,
    );
    if game.pending_kind.is_some() {
        add_text(
            commands,
            status,
            if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                format!("指定颜色摸牌 ×{}", game.pending_draw)
            } else {
                format!("累计罚牌 +{}", game.pending_draw)
            },
            17.0,
            DANGER,
            assets,
        );
    }
    if game.pending_skip > 0 {
        add_text(
            commands,
            status,
            format!("累计禁手 ×{}", game.pending_skip),
            17.0,
            DANGER,
            assets,
        );
    }
}

fn add_uno_own_area(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    own: &UnoPlayerState,
    ui: &UiState,
    assets: &UiAssets,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
) {
    let info = add_panel(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(22),
            bottom: px(15),
            width: px(175),
            height: px(76),
            padding: UiRect::all(px(10)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(4),
            ..default()
        },
        PANEL.with_alpha(0.94),
        PanelSkin::Section,
        assets,
    );
    let selecting_self = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ForceTrade { player }) if player == game.you
    );
    let self_selected = selecting_self && ui.uno_swap_targets.contains(&game.you);
    if selecting_self {
        commands.entity(info).insert((
            Button,
            UiAction::ToggleUnoSwapTarget(game.you),
            UnoSwapTargetPanel {
                selected: self_selected,
            },
            BackgroundColor(if self_selected {
                Color::BLACK.with_alpha(0.76)
            } else {
                ACCENT.mix(&PANEL, 0.48).with_alpha(0.97)
            }),
            Outline::new(px(2.0), px(1.0), ACCENT.with_alpha(0.92)),
            BoxShadow::new(ACCENT.with_alpha(0.32), px(0), px(0), px(2), px(9)),
        ));
    }
    commands.entity(info).insert(PlayerAvatarAnchor(game.you));
    if game.current_player == Some(game.you) && matches!(game.phase, UnoPhaseView::Playing) {
        add_turn_border_trace(
            commands,
            info,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Uno, game.match_id, game.you),
        );
    }
    add_text(
        commands,
        info,
        format!("你 · {}", own.name),
        15.0,
        TEXT,
        assets,
    );
    if self_selected {
        add_uno_swap_selected_label(commands, info, assets);
    }
    add_uno_skip_overlay(commands, info, uno_skip_count(game, own), assets);
    add_text(
        commands,
        info,
        format!("{} 张牌", game.your_hand.len()),
        13.0,
        MUTED,
        assets,
    );

    let hand = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(205),
            right: px(205),
            bottom: px(4),
            height: px(146),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::Center,
            overflow: Overflow::visible(),
            ..default()
        },
        None,
    );
    let count = game.your_hand.len();
    let reveal = if count <= 1 {
        78.0
    } else {
        (820.0 / count as f32).clamp(24.0, 72.0)
    };
    for (index, card) in game.your_hand.iter().copied().enumerate() {
        let playing = matches!(game.phase, UnoPhaseView::Playing);
        let playable = uno_card_is_playable(game, card);
        let selectable_for_swap = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneGive { player, .. }) if player == game.you
        );
        let interactive = playable || selectable_for_swap;
        let jump_selected = game.your_jump_in_card == Some(card);
        let selected = playing && ui.selected_uno.contains(&card);
        let animation = if playing {
            ui.uno_card_animations
                .get(&card)
                .copied()
                .unwrap_or_default()
        } else {
            CardAnimationState::default()
        };
        let mut entity = commands.spawn((
            Node {
                width: px(if index + 1 == count { 78.0 } else { reveal }),
                height: px(126),
                flex_shrink: 0.0,
                align_items: AlignItems::FlexStart,
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ));
        let extension_help = uno_extension_card_help(card.face());
        if interactive {
            entity.insert((
                Button,
                UnoHandCardButton,
                UiAction::ToggleUnoCard(card),
                ButtonTint {
                    normal: Color::WHITE,
                    hovered: Color::srgb(1.0, 0.92, 0.66),
                    pressed: Color::srgb(0.78, 0.84, 0.72),
                },
            ));
        } else if extension_help.is_some() {
            entity.insert(Button);
        }
        let slot = entity.id();
        commands.entity(hand).add_child(slot);
        let face = commands
            .spawn((
                UnoHandCardVisual {
                    button: slot,
                    card,
                    selected,
                    hover_amount: animation.face_hover_amount,
                    selected_amount: animation.selected_amount,
                },
                Node {
                    width: px(78),
                    height: px(122),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    ..default()
                },
                UiTransform::from_translation(Val2::px(
                    0.0,
                    -(animation.face_hover_amount * 10.0 + animation.selected_amount * 22.0),
                )),
                ImageNode::new(uno_card_handle(assets, card)).with_color(
                    if interactive || jump_selected || !playing {
                        Color::WHITE
                    } else {
                        Color::srgba(0.58, 0.58, 0.58, 0.86)
                    },
                ),
                BoxShadow::new(Color::BLACK.with_alpha(0.42), px(2), px(4), px(0), px(5)),
                BorderColor::all(if selected { ACCENT } else { Color::NONE }),
                Outline {
                    width: px(if selected { 2.0 } else { 0.0 }),
                    offset: px(0),
                    color: ACCENT.with_alpha(0.8),
                },
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(slot).add_child(face);
        if let Some((title, description)) = extension_help {
            commands
                .entity(slot)
                .insert(UnoExtensionCardHelp { title, description });
        }
    }
}

pub(in crate::app) const fn uno_extension_card_help(
    face: UnoFace,
) -> Option<(&'static str, &'static str)> {
    match face {
        UnoFace::SwapOne => Some((
            "交换一张",
            "随机取得一名玩家的一张牌，再从当前手牌中选择一张交还。",
        )),
        UnoFace::RefreshHand => {
            Some(("刷新手牌", "将剩余手牌放到弃牌堆底部，再摸取相同数量的牌。"))
        }
        UnoFace::WildForceTrade => {
            Some(("指定换手", "选择两名玩家交换全部手牌，完成后选择后续颜色。"))
        }
        UnoFace::WildPassHands => Some((
            "顺序传手",
            "所有玩家按当前方向传递全部手牌，完成后选择后续颜色。",
        )),
        UnoFace::ReverseDrawTwo => Some((
            "反转摸二",
            "立即改变方向，并让新方向的下一名玩家累计摸两张牌。",
        )),
        UnoFace::ReverseSkip => Some((
            "反转禁手",
            "立即改变方向，并让新方向的下一名玩家被禁手一轮。",
        )),
        UnoFace::WildPowerReverse => Some((
            "强力反转",
            "改变方向并选择后续颜色，然后由你立即再行动一次。",
        )),
        UnoFace::WildNoU => Some((
            "罚牌反弹",
            "受到摸牌惩罚时将累计罚牌退给上一名罚牌者；平时作为万能反转牌使用。",
        )),
        UnoFace::StackOne => Some((
            "堆叠 +1",
            "按当前颜色打出，使下家累计摸一张；罚牌链中也必须匹配当前颜色。",
        )),
        UnoFace::StackTwo => Some((
            "堆叠 +2",
            "按当前颜色打出，使下家累计摸两张；罚牌链中也必须匹配当前颜色。",
        )),
        UnoFace::WildStackThree => {
            Some(("万能堆叠 +3", "可随时打出并选择颜色，使下家累计摸三张。"))
        }
        UnoFace::WildStackNumber => Some((
            "万能随机堆叠",
            "选择颜色后从摸牌堆翻牌，直到出现数字牌，并将该数字加入累计罚牌。",
        )),
        UnoFace::DrawFour => Some(("摸四", "使下一名玩家累计摸四张；可压在 +2 或 +4 上。")),
        UnoFace::DrawFive => Some((
            "摸五",
            "使下一名玩家累计摸五张；启用功能牌堆叠时可继续累计。",
        )),
        UnoFace::SkipEveryone => Some(("跳过所有人", "跳过所有其他玩家，由你立即再行动一次。")),
        UnoFace::Flip => Some((
            "翻面",
            "翻转摸牌堆、弃牌堆和所有玩家手牌，并改用另一面的牌面继续游戏。",
        )),
        UnoFace::WildDrawColor => Some((
            "指定颜色摸牌",
            "选择一种颜色；下一名玩家持续摸牌，直到摸到该颜色。",
        )),
        UnoFace::DiscardAll => Some(("全部弃牌", "同时弃掉手中所有与此牌同色的牌。")),
        UnoFace::WildReverseDrawFour => Some((
            "反转摸四",
            "改变方向，选择颜色，并让新方向的下一名玩家累计摸四张。",
        )),
        UnoFace::WildDrawSix => Some(("万能摸六", "选择颜色，使下一名玩家累计摸六张。")),
        UnoFace::WildDrawTen => Some(("万能摸十", "选择颜色，使下一名玩家累计摸十张。")),
        UnoFace::WildColorRoulette => Some((
            "颜色轮盘",
            "下一名玩家选择颜色，持续翻牌并收下所有牌，直到出现该颜色。",
        )),
        UnoFace::Number(_)
        | UnoFace::DrawOne
        | UnoFace::DrawTwo
        | UnoFace::Reverse
        | UnoFace::Skip
        | UnoFace::Wild
        | UnoFace::WildDrawTwo
        | UnoFace::WildDrawFour => None,
    }
}

fn spawn_uno_extension_card_tooltip(
    commands: &mut Commands,
    layer: Entity,
    assets: &UiAssets,
) -> Entity {
    let tooltip = spawn_node(
        commands,
        layer,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: px(198),
            padding: UiRect::axes(px(11), px(9)),
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(PANEL.with_alpha(0.78)),
    );
    commands.entity(tooltip).insert((
        Visibility::Hidden,
        BorderColor::all(ACCENT.with_alpha(0.24)),
        BoxShadow::new(Color::BLACK.with_alpha(0.30), px(2), px(4), px(0), px(8)),
        GlobalZIndex(1900),
        FocusPolicy::Pass,
    ));
    let title = add_text(commands, tooltip, "", 13.0, TEXT.with_alpha(0.90), assets);
    let description = add_text(commands, tooltip, "", 11.5, MUTED.with_alpha(0.88), assets);
    commands
        .entity(tooltip)
        .insert(UnoExtensionCardHelpOverlay { title, description });
    tooltip
}

pub(in crate::app) fn sync_uno_extension_card_help(
    mut commands: Commands,
    assets: Res<UiAssets>,
    cards: Query<(
        &Interaction,
        &UnoExtensionCardHelp,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    mut overlays: Query<(&UnoExtensionCardHelpOverlay, &mut Node, &mut Visibility)>,
    mut texts: Query<&mut Text>,
) {
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let Ok((overlay, mut node, mut visibility)) = overlays.single_mut() else {
        spawn_uno_extension_card_tooltip(&mut commands, layer, &assets);
        return;
    };
    let Some((_, help, card_node, card_transform)) = cards.iter().find(|(interaction, ..)| {
        matches!(interaction, Interaction::Hovered | Interaction::Pressed)
    }) else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(center) = uno_anchor_in_layer(card_node, card_transform, layer_node, layer_transform)
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    let size = card_node.size() * card_node.inverse_scale_factor();
    node.left = px(center.x - size.x * 0.5 + 84.0);
    node.top = px(center.y - size.y * 0.5 + 12.0);
    if let Ok(mut title) = texts.get_mut(overlay.title)
        && title.0 != help.title
    {
        title.0 = help.title.to_owned();
    }
    if let Ok(mut description) = texts.get_mut(overlay.description)
        && description.0 != help.description
    {
        description.0 = help.description.to_owned();
    }
    *visibility = Visibility::Visible;
}

fn add_uno_actions(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    ui: &UiState,
    assets: &UiAssets,
) {
    if !matches!(game.phase, UnoPhaseView::Playing)
        || game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.eliminated)
    {
        return;
    }
    if let Some(pending) = game.pending_swap {
        if matches!(pending, UnoPendingSwapView::SwapOneGive { player, .. } if player == game.you) {
            let actions = spawn_node(
                commands,
                table,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(390),
                    bottom: px(UNO_ACTION_AREA_BOTTOM),
                    width: px(500),
                    height: px(UNO_ACTION_AREA_HEIGHT),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                None,
            );
            if ui.selected_uno.len() == 1 {
                add_action_button(
                    commands,
                    actions,
                    "交出选中的牌",
                    UiAction::SubmitUnoCard,
                    ButtonKind::Primary,
                    assets,
                );
            } else {
                add_disabled_action_button(commands, actions, "请选择一张要交出的牌", assets);
            }
        }
        return;
    }
    let jump_in = game.your_jump_in_card;
    if game.current_player != Some(game.you) && jump_in.is_none() {
        return;
    }
    let actions = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: px(390),
            bottom: px(UNO_ACTION_AREA_BOTTOM),
            width: px(500),
            height: px(UNO_ACTION_AREA_HEIGHT),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    if let Some(card) = jump_in {
        add_action_button(
            commands,
            actions,
            "抢出",
            UiAction::UnoJumpIn(card),
            ButtonKind::Primary,
            assets,
        );
        return;
    }
    if game.current_color.is_none() {
        return;
    }
    let own_skips = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.skipped_turns);
    let selected = match ui
        .selected_uno
        .iter()
        .copied()
        .collect::<Vec<_>>()
        .as_slice()
    {
        [card] if uno_card_is_playable(game, *card) => Some((*card, 1)),
        [first, second]
            if uno_card_is_playable(game, *first)
                && uno_pair_for_selection(game, *first) == Some(*second) =>
        {
            Some((*first, 2))
        }
        _ => None,
    };
    if game.uno_declared.contains(&game.you) && selected.is_none() {
        return;
    }
    if let Some((card, card_count)) = selected {
        add_action_button(
            commands,
            actions,
            if matches!(
                card.face(),
                UnoFace::Wild
                    | UnoFace::WildDrawTwo
                    | UnoFace::WildDrawFour
                    | UnoFace::WildDrawColor
                    | UnoFace::WildPowerReverse
                    | UnoFace::WildNoU
                    | UnoFace::WildStackThree
                    | UnoFace::WildStackNumber
                    | UnoFace::WildReverseDrawFour
                    | UnoFace::WildDrawSix
                    | UnoFace::WildDrawTen
            ) {
                "出牌并选色"
            } else if card_count == 2 {
                "一次打出 ×2"
            } else {
                "出牌"
            },
            UiAction::SubmitUnoCard,
            ButtonKind::Primary,
            assets,
        );
    } else if game.pending_kind.is_some() {
        add_action_button(
            commands,
            actions,
            &if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                "接受指定颜色摸牌".to_owned()
            } else {
                format!("接受 +{}", game.pending_draw)
            },
            UiAction::UnoAcceptDrawPenalty,
            ButtonKind::Warning,
            assets,
        );
        if game.challenge_offender.is_some() {
            add_action_button(
                commands,
                actions,
                if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawColor) {
                    "质疑指定颜色摸牌"
                } else if game.pending_kind == Some(UnoPendingDrawKind::FlipWildDrawTwo) {
                    "质疑万能 +2"
                } else {
                    "质疑 +4"
                },
                UiAction::UnoChallengeDrawFour,
                ButtonKind::Pass,
                assets,
            );
        }
    } else if game.pending_skip > 0 || own_skips > 0 {
        add_action_button(
            commands,
            actions,
            &format!("接受禁手 ×{}", game.pending_skip + own_skips),
            UiAction::UnoResolveSkip,
            ButtonKind::Pass,
            assets,
        );
    } else if game.your_drawn_card.is_some() {
        if !game.rules.is_no_mercy() || !game.rules.no_mercy.draw_until_playable {
            add_action_button(
                commands,
                actions,
                "结束回合",
                UiAction::UnoPassAfterDraw,
                ButtonKind::Secondary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "必须打出摸到的牌", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            "摸牌",
            UiAction::UnoDrawCard,
            ButtonKind::Secondary,
            assets,
        );
    }
}

fn add_uno_callout_actions(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    assets: &UiAssets,
) {
    if game.pending_swap.is_some()
        || game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .is_some_and(|player| player.eliminated)
    {
        return;
    }
    let targets = game
        .uno_exposed
        .iter()
        .copied()
        .filter(|target| *target != game.you)
        .collect::<Vec<_>>();
    let own_skips = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.skipped_turns);
    let can_declare_before_play = game.current_player == Some(game.you)
        && game.your_hand.len() == 2
        && game.pending_kind.is_none()
        && game.pending_skip == 0
        && own_skips == 0
        && game
            .your_hand
            .iter()
            .copied()
            .any(|card| uno_card_is_playable(game, card));
    let can_recover_after_play = game.your_hand.len() == 1 && game.uno_exposed.contains(&game.you);
    let can_call = game.rules.uno_callout()
        && matches!(game.phase, UnoPhaseView::Playing)
        && !game.uno_declared.contains(&game.you)
        && (can_declare_before_play || can_recover_after_play);
    let callouts = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            right: px(24),
            bottom: px(16),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: px(5),
            ..default()
        },
        None,
    );
    commands.entity(callouts).insert(GlobalZIndex(1700));
    for target in targets {
        let name = game
            .players
            .iter()
            .find(|player| player.id == target)
            .map(|player| player.name.as_str())
            .unwrap_or("玩家");
        add_subtle_uno_button(
            commands,
            callouts,
            &format!("检举 {name}"),
            Some(UiAction::UnoReport(target)),
            DANGER,
            assets,
        );
    }
    add_subtle_uno_button(
        commands,
        callouts,
        "UNO!",
        can_call.then_some(UiAction::UnoCall),
        READY,
        assets,
    );
}

fn add_subtle_uno_button(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    action: Option<UiAction>,
    color: Color,
    assets: &UiAssets,
) {
    let mut button = commands.spawn((
        Node {
            min_width: px(90),
            height: px(30),
            padding: UiRect::horizontal(px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            ..default()
        },
        BackgroundColor(color.with_alpha(0.56)),
        BorderColor::all(color.with_alpha(0.72)),
    ));
    if let Some(action) = action {
        button.insert((
            Button,
            action,
            ButtonTint {
                normal: color.with_alpha(0.56),
                hovered: color.with_alpha(0.56),
                pressed: color.with_alpha(0.56),
            },
        ));
    }
    let button = button.id();
    commands.entity(parent).add_child(button);
    add_text(commands, button, label, 12.0, TEXT, assets);
}

fn add_initial_color_choice(
    commands: &mut Commands,
    parent: Entity,
    game: &UnoSnapshot,
    assets: &UiAssets,
) {
    let choosing_player = match game.pending_swap {
        Some(
            UnoPendingSwapView::ChooseColor { player }
            | UnoPendingSwapView::ColorRoulette { player },
        ) => Some(player),
        _ => game.current_player,
    };
    let name = choosing_player
        .and_then(|id| game.players.iter().find(|player| player.id == id))
        .map(|player| player.name.as_str())
        .unwrap_or("玩家");
    let own_turn = choosing_player == Some(game.you);
    let after_swap = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ChooseColor { .. })
    );
    let color_roulette = matches!(
        game.pending_swap,
        Some(UnoPendingSwapView::ColorRoulette { .. })
    );
    add_color_choice_overlay(
        commands,
        parent,
        if own_turn && color_roulette {
            "选择一种颜色并摸牌，直到翻出该颜色".to_owned()
        } else if own_turn && after_swap {
            "换牌完成，请选择后续颜色".to_owned()
        } else if own_turn {
            "起始牌是万能牌，请选择颜色".to_owned()
        } else if color_roulette {
            format!("等待 {name} 选择颜色轮盘目标色")
        } else if after_swap {
            format!("等待 {name} 选择后续颜色")
        } else {
            format!("等待 {name} 选择起始颜色")
        },
        own_turn.then_some(None),
        game.flip_side,
        assets,
    );
}

fn add_play_color_choice(
    commands: &mut Commands,
    parent: Entity,
    flip_side: Option<leocard_uno::FlipSide>,
    card: UnoCard,
    assets: &UiAssets,
) {
    add_color_choice_overlay(
        commands,
        parent,
        "选择后续颜色".to_owned(),
        Some(Some(card)),
        flip_side,
        assets,
    );
}

fn add_color_choice_overlay(
    commands: &mut Commands,
    parent: Entity,
    title: String,
    choice: Option<Option<UnoCard>>,
    flip_side: Option<leocard_uno::FlipSide>,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.58)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2050), FocusPolicy::Block));
    let panel = add_panel(
        commands,
        overlay,
        Node {
            width: px(480),
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(18),
            ..default()
        },
        PANEL,
        PanelSkin::Popup,
        assets,
    );
    add_section_title(commands, panel, title, assets);
    if let Some(card) = choice {
        let color_row = spawn_node(
            commands,
            panel,
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: px(12),
                ..default()
            },
            None,
        );
        let colors = match flip_side {
            Some(leocard_uno::FlipSide::Dark) => UnoColor::DARK,
            Some(leocard_uno::FlipSide::Light) | None => UnoColor::LIGHT,
        };
        for color in colors {
            let button = commands
                .spawn((
                    Button,
                    card.map_or(UiAction::UnoChooseInitialColor(color), |card| {
                        UiAction::UnoPlayCard(card, Some(color))
                    }),
                    ButtonTint {
                        normal: uno_ui_color(color),
                        hovered: uno_ui_color(color).mix(&Color::WHITE, 0.22),
                        pressed: uno_ui_color(color).mix(&Color::BLACK, 0.22),
                    },
                    Node {
                        width: px(78),
                        height: px(78),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    BackgroundColor(uno_ui_color(color)),
                    BorderColor::all(Color::WHITE.with_alpha(0.78)),
                ))
                .id();
            commands.entity(color_row).add_child(button);
            add_text(
                commands,
                button,
                color.to_string(),
                15.0,
                Color::WHITE,
                assets,
            );
        }
        if card.is_some() {
            add_action_button(
                commands,
                panel,
                "取消",
                UiAction::CloseUnoColorChoice,
                ButtonKind::Secondary,
                assets,
            );
        }
    } else {
        add_text(commands, panel, "正在等待其他玩家……", 14.0, MUTED, assets);
    }
}

fn add_uno_summary(
    commands: &mut Commands,
    table: Entity,
    game: &UnoSnapshot,
    winner: PlayerId,
    results: &[leocard_protocol::UnoPlayerResult],
    reference_changes: &[leocard_protocol::PlayerReferenceChange],
    assets: &UiAssets,
    avatars: &AvatarImages,
    animation: &GameSummaryAnimation,
) {
    let modal_visual = summary_modal_visual(animation.elapsed);
    let modal = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(25),
            right: percent(25),
            top: percent(3),
            min_height: px(380),
            padding: UiRect::all(px(26)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        None,
    );
    commands.entity(modal).insert((
        GameSummaryModal,
        UiTransform::from_translation(Val2::px(0.0, modal_visual.offset_y)),
        GlobalZIndex(1200),
        FocusPolicy::Block,
        if animation.elapsed >= 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    let texture = decorate_panel_skin(commands, modal, PanelSkin::Window, assets);
    commands.entity(texture).insert(GameSummaryPanelTexture);
    let winner_name = game
        .players
        .iter()
        .find(|player| player.id == winner)
        .map(|player| player.name.as_str())
        .unwrap_or("玩家");
    add_animated_summary_text(
        commands,
        modal,
        "本局结算",
        26.0,
        ACCENT,
        0.0,
        animation.elapsed,
        assets,
    );
    add_animated_summary_text(
        commands,
        modal,
        format!("{winner_name} 率先出完手牌"),
        14.0,
        TEXT,
        0.0,
        animation.elapsed,
        assets,
    );

    let score_list = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        },
        None,
    );
    let mut ordered = results.to_vec();
    ordered.sort_by_key(|result| result.placement);
    for (index, result) in ordered.iter().enumerate() {
        let Some(player) = game
            .players
            .iter()
            .find(|player| player.id == result.player)
        else {
            continue;
        };
        let delay = SUMMARY_ROW_START_DELAY + index as f32 * SUMMARY_ROW_INTERVAL;
        let row_progress = summary_row_progress(animation.elapsed, delay);
        let row = spawn_node(
            commands,
            score_list,
            Node {
                width: percent(100),
                min_height: px(38),
                padding: UiRect::axes(px(10), px(4)),
                align_items: AlignItems::Center,
                column_gap: px(10),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            Some(PANEL_ALT.with_alpha(0.82 * row_progress)),
        );
        commands.entity(row).insert((
            GameSummaryRow { delay },
            UiTransform::from_translation(Val2::px(0.0, 12.0 * (1.0 - row_progress))),
            if row_progress > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ));
        let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
        add_ready_avatar(
            commands,
            row,
            &player.name,
            avatar,
            28.0,
            player.ready,
            assets,
        );
        let name = spawn_node(
            commands,
            row,
            Node {
                flex_grow: 1.0,
                min_width: px(0),
                ..default()
            },
            None,
        );
        add_animated_summary_text(
            commands,
            name,
            format!("第 {} 名 · {}", result.placement, player.name),
            16.0,
            if result.player == game.you {
                ACCENT
            } else {
                TEXT
            },
            delay,
            animation.elapsed,
            assets,
        );
        add_animated_summary_text(
            commands,
            row,
            if player.eliminated {
                "状态"
            } else {
                "手牌"
            },
            13.0,
            MUTED,
            delay,
            animation.elapsed,
            assets,
        );
        if player.eliminated {
            add_animated_summary_text(
                commands,
                row,
                "已淘汰",
                17.0,
                DANGER,
                delay,
                animation.elapsed,
                assets,
            );
        } else {
            let displayed =
                animated_summary_score(animation.elapsed, u32::from(result.hand_score), delay);
            let score_text = add_animated_summary_text(
                commands,
                row,
                format!("{displayed} 分"),
                19.0,
                ACCENT,
                delay,
                animation.elapsed,
                assets,
            );
            commands.entity(score_text).insert(AnimatedSummaryScore {
                target: u32::from(result.hand_score),
                delay,
            });
        }
        let delta = reference_changes
            .iter()
            .find(|change| change.player == result.player)
            .map_or(0, |change| change.delta);
        add_animated_summary_text(
            commands,
            row,
            format!("{delta:+}"),
            14.0,
            if delta >= 0 { READY } else { DANGER },
            delay,
            animation.elapsed,
            assets,
        );
    }

    let actions_delay = SUMMARY_ROW_START_DELAY
        + ordered.len() as f32 * SUMMARY_ROW_INTERVAL
        + SUMMARY_ACTIONS_EXTRA_DELAY;
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(52),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    commands.entity(actions).insert((
        GameSummaryActions {
            delay: actions_delay,
        },
        if animation.elapsed >= actions_delay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
    ));
    add_action_button(
        commands,
        actions,
        "退出游戏",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if game.you == game.host {
        add_action_button(
            commands,
            actions,
            "返回房间",
            UiAction::ReturnToLobby,
            ButtonKind::Warning,
            assets,
        );
    }
    let ready = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .is_some_and(|player| player.ready);
    if ready {
        add_disabled_action_button(commands, actions, "已准备", assets);
    } else {
        add_action_button(
            commands,
            actions,
            "再来一局",
            UiAction::PlayAgain,
            ButtonKind::Primary,
            assets,
        );
    }
}

fn uno_card_handle(assets: &UiAssets, card: UnoCard) -> Handle<Image> {
    assets
        .uno_cards
        .get(&(card.color(), card.face()))
        .cloned()
        .expect("所有 UNO 牌面都应预加载")
}

fn uno_card_is_playable(game: &UnoSnapshot, card: UnoCard) -> bool {
    if game.current_player != Some(game.you)
        || game.pending_swap.is_some()
        || game.current_color.is_none()
        || !matches!(game.phase, UnoPhaseView::Playing)
        || game
            .your_drawn_card
            .is_some_and(|drawn_card| drawn_card != card)
    {
        return false;
    }
    let own_skips = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .map_or(0, |player| player.skipped_turns);
    if own_skips > 0 {
        return false;
    }
    if game.pending_skip > 0 {
        return if game.rules.is_flip() {
            game.rules.flip.action_stacking
                && card.face() == game.discard_top.face()
                && matches!(card.face(), UnoFace::Skip | UnoFace::SkipEveryone)
        } else {
            game.rules.action_stacking
                && matches!(card.face(), UnoFace::Skip | UnoFace::ReverseSkip)
        };
    }
    if game.pending_kind.is_some() {
        if game.rules.is_no_mercy() {
            return match game.pending_kind {
                Some(UnoPendingDrawKind::NoMercy(minimum)) => card
                    .face()
                    .draw_value()
                    .is_some_and(|value| value >= minimum),
                _ => false,
            };
        }
        if game.rules.is_flip() {
            if !game.rules.flip.action_stacking {
                return false;
            }
            return matches!(
                (game.pending_kind, card.face()),
                (
                    Some(UnoPendingDrawKind::FlipDrawOne),
                    UnoFace::DrawOne | UnoFace::WildDrawTwo
                ) | (
                    Some(UnoPendingDrawKind::FlipWildDrawTwo),
                    UnoFace::WildDrawTwo
                ) | (Some(UnoPendingDrawKind::FlipDrawFive), UnoFace::DrawFive)
                    | (
                        Some(UnoPendingDrawKind::FlipWildDrawColor),
                        UnoFace::WildDrawColor
                    )
            );
        }
        if !game.rules.action_stacking {
            return false;
        }
        return match (game.pending_kind, card.face()) {
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::DrawTwo | UnoFace::ReverseDrawTwo)
            | (Some(UnoPendingDrawKind::WildDrawFour), UnoFace::WildDrawFour) => true,
            (_, UnoFace::WildNoU) => true,
            (_, UnoFace::StackOne | UnoFace::StackTwo) => card.color() == game.current_color,
            (_, UnoFace::WildStackThree | UnoFace::WildStackNumber) => true,
            (Some(UnoPendingDrawKind::DrawTwo), UnoFace::WildDrawFour) => true,
            _ => false,
        };
    }
    card.face().is_wild()
        || card.color() == game.current_color
        || uno_faces_match(card.face(), game.discard_top.face())
}

fn uno_faces_match(left: UnoFace, right: UnoFace) -> bool {
    if left == right {
        return !matches!(left, UnoFace::StackOne | UnoFace::StackTwo);
    }
    matches!(
        (left, right),
        (UnoFace::ReverseDrawTwo, UnoFace::Reverse | UnoFace::DrawTwo)
            | (UnoFace::Reverse | UnoFace::DrawTwo, UnoFace::ReverseDrawTwo)
            | (UnoFace::ReverseSkip, UnoFace::Reverse | UnoFace::Skip)
            | (UnoFace::Reverse | UnoFace::Skip, UnoFace::ReverseSkip)
    )
}

pub(in crate::app) fn uno_pair_for_selection(
    game: &UnoSnapshot,
    selected: UnoCard,
) -> Option<UnoCard> {
    let jump_in = if game.rules.is_flip() {
        game.rules.flip.jump_in
    } else {
        game.rules.is_classic() && game.rules.jump_in
    };
    if !jump_in
        || game.uno_declared.contains(&game.you)
        || selected.color().is_none()
        || selected.face().is_extension()
    {
        return None;
    }
    game.your_hand.iter().copied().find(|card| {
        *card != selected && card.color() == selected.color() && card.face() == selected.face()
    })
}

pub(in crate::app) fn toggle_uno_selection(
    game: Option<&UnoSnapshot>,
    selected: &mut HashSet<UnoCard>,
    card: UnoCard,
) {
    if selected.remove(&card) {
        return;
    }
    if selected.len() == 1 {
        let first = *selected.iter().next().unwrap();
        if game.is_some_and(|game| uno_pair_for_selection(game, first) == Some(card)) {
            selected.insert(card);
            return;
        }
    }
    selected.clear();
    selected.insert(card);
}

fn uno_ui_color(color: UnoColor) -> Color {
    match color {
        UnoColor::Red => Color::srgb(0.91, 0.18, 0.16),
        UnoColor::Yellow => Color::srgb(0.96, 0.72, 0.08),
        UnoColor::Green => Color::srgb(0.10, 0.67, 0.28),
        UnoColor::Blue => Color::srgb(0.08, 0.42, 0.86),
        UnoColor::Pink => Color::srgb(0.91, 0.24, 0.58),
        UnoColor::Teal => Color::srgb(0.06, 0.62, 0.62),
        UnoColor::Orange => Color::srgb(0.96, 0.43, 0.10),
        UnoColor::Purple => Color::srgb(0.48, 0.25, 0.76),
    }
}

pub(in crate::app) fn uno_should_show_reverse_effect(
    card: UnoCard,
    play_index: u8,
    play_count: u8,
) -> bool {
    play_index == 0
        && match card.face() {
            UnoFace::Reverse => play_count % 2 == 1,
            UnoFace::ReverseDrawTwo
            | UnoFace::ReverseSkip
            | UnoFace::WildPowerReverse
            | UnoFace::WildNoU
            | UnoFace::WildReverseDrawFour => true,
            _ => false,
        }
}

/// UNO 手牌沿用其他游戏的柔和抬升、渐变描边与阴影，不用突兀的离散跳变。
pub(in crate::app) fn animate_uno_hand_cards(
    time: Res<Time>,
    mut ui: ResMut<UiState>,
    buttons: Query<&Interaction, With<Button>>,
    mut cards: Query<(
        &mut UnoHandCardVisual,
        &mut UiTransform,
        &mut Outline,
        &mut BoxShadow,
        &mut BorderColor,
    )>,
) {
    let response = 1.0 - (-14.0 * time.delta_secs()).exp();
    let pulse = 0.76 + 0.24 * (time.elapsed_secs() * 6.5).sin();
    for (mut visual, mut transform, mut outline, mut shadow, mut border) in &mut cards {
        let selected = ui.selected_uno.contains(&visual.card);
        let hovered = buttons.get(visual.button).is_ok_and(|interaction| {
            matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
        });
        let hover_target = f32::from(hovered);
        let selected_target = f32::from(selected);
        visual.hover_amount += (hover_target - visual.hover_amount) * response;
        visual.selected_amount += (selected_target - visual.selected_amount) * response;
        visual.selected = selected;

        let glow = (visual.selected_amount * (0.76 + pulse * 0.24)).clamp(0.0, 1.0);
        transform.translation = Val2::px(
            0.0,
            -(visual.hover_amount * 10.0 + visual.selected_amount * 22.0),
        );
        transform.scale = Vec2::splat(1.0 + visual.hover_amount * 0.025);
        outline.width = px(glow * 2.3);
        outline.color = ACCENT.with_alpha(glow * 0.9);
        border.set_all(ACCENT.with_alpha(visual.selected_amount));
        if let Some(style) = shadow.0.first_mut() {
            style.color = if visual.selected_amount > 0.01 {
                ACCENT.with_alpha(glow * 0.5)
            } else {
                Color::BLACK.with_alpha(0.42)
            };
            style.spread_radius = px(glow * 1.6);
            style.blur_radius = px(5.0 + glow * 8.0);
        }
        ui.uno_card_animations.insert(
            visual.card,
            CardAnimationState {
                face_hover_amount: visual.hover_amount,
                selected_amount: visual.selected_amount,
                ..default()
            },
        );
    }
}

pub(in crate::app) fn animate_uno_swap_target_panels(
    time: Res<Time>,
    mut panels: Query<(
        &UnoSwapTargetPanel,
        &mut BackgroundColor,
        &mut Outline,
        &mut BoxShadow,
    )>,
) {
    let pulse = 0.5 + 0.5 * (time.elapsed_secs() * 4.8).sin();
    for (target, mut background, mut outline, mut shadow) in &mut panels {
        if target.selected {
            background.0 = Color::BLACK.with_alpha(0.78);
            outline.width = px(2.4);
            outline.color = ACCENT.with_alpha(0.96);
        } else {
            background.0 = ACCENT.mix(&PANEL, 0.42 + pulse * 0.12).with_alpha(0.97);
            outline.width = px(2.0 + pulse * 0.8);
            outline.color = ACCENT.with_alpha(0.68 + pulse * 0.28);
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = ACCENT.with_alpha(if target.selected {
                0.42
            } else {
                0.24 + pulse * 0.22
            });
            style.blur_radius = px(if target.selected {
                11.0
            } else {
                8.0 + pulse * 6.0
            });
            style.spread_radius = px(1.0 + pulse * 1.5);
        }
    }
}

pub(in crate::app) fn sync_uno_presentation(
    mut client: Option<ResMut<ClientResource>>,
    mut presentation: ResMut<UnoPresentationState>,
) {
    let Some(client) = client.as_deref_mut() else {
        presentation.events.clear();
        return;
    };
    presentation.events.extend(client.0.take_uno_events());
    if client.0.model().uno_game().is_none() {
        presentation.events.clear();
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::app) fn spawn_uno_presentation_effects(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    assets: Res<UiAssets>,
    mut palette_materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut presentation: ResMut<UnoPresentationState>,
    mut audio: ResMut<UnoAudioState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    players: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    draws: Query<(&ComputedNode, &UiGlobalTransform), With<UnoDrawPileAnchor>>,
    discards: Query<(&ComputedNode, &UiGlobalTransform), With<UnoDiscardPileAnchor>>,
) {
    let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    else {
        presentation.events.clear();
        return;
    };
    let Ok((layer, layer_node, layer_transform)) = layers.single() else {
        return;
    };
    if layer_node.size().min_element() <= 1.0 {
        return;
    }
    let Ok((draw_node, draw_transform)) = draws.single() else {
        return;
    };
    let Ok((discard_node, discard_transform)) = discards.single() else {
        return;
    };
    let Some(draw_position) =
        uno_anchor_in_layer(draw_node, draw_transform, layer_node, layer_transform)
    else {
        return;
    };
    let Some(discard_position) =
        uno_anchor_in_layer(discard_node, discard_transform, layer_node, layer_transform)
    else {
        return;
    };
    let discard_top_pose = uno_discard_pose(game.discard_top);
    // `render_ui` 会在收到权威快照时重建牌桌；同一帧中新节点尚未经过
    // Bevy 的布局阶段，ComputedNode/UiGlobalTransform 仍指向左上角。等到下一帧
    // 所有锚点拥有真实尺寸后再消费事件，否则整组动画会被画在 (0, 0)。
    if !game.players.iter().all(|player| {
        uno_player_anchor_in_layer(player.id, layer_node, layer_transform, &players).is_some()
    }) {
        return;
    }

    while let Some(event) = presentation.events.pop_front() {
        audio.queue_event(&event, game.you);
        match event {
            UnoEvent::CardPlayed {
                player,
                card,
                chosen_color,
                play_index,
                play_count,
            } => {
                let source =
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(draw_position);
                let card_pose = uno_discard_pose(card);
                let card_target = discard_position
                    + Vec2::new(
                        card_pose.0 - discard_top_pose.0,
                        card_pose.1 - discard_top_pose.1,
                    );
                spawn_uno_flying_card(
                    &mut commands,
                    layer,
                    uno_card_handle(&assets, card),
                    Some(card),
                    source,
                    card_target,
                    0.0,
                    false,
                    card.copy() as usize,
                    card_pose.2,
                );
                if uno_should_show_reverse_effect(card, play_index, play_count) {
                    spawn_uno_reverse_effect(
                        &mut commands,
                        layer,
                        layer_node,
                        layer_transform,
                        game,
                        &players,
                        card.color()
                            .or(chosen_color)
                            .map(uno_ui_color)
                            .expect("反转牌应带有牌色或已选择的后续颜色"),
                    );
                }
                if play_index == 0
                    && let Some(color) = chosen_color
                {
                    spawn_uno_palette_effect(
                        &mut commands,
                        layer,
                        discard_position,
                        color,
                        &mut palette_materials,
                    );
                }
            }
            UnoEvent::CardsDrawn {
                player,
                count,
                penalty,
            } => {
                let interval = if !penalty && count > 1 { 0.18 } else { 0.045 };
                spawn_uno_draw_cards_with_interval(
                    &mut commands,
                    layer,
                    draw_position,
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position),
                    count,
                    0.0,
                    interval,
                    &assets,
                );
            }
            UnoEvent::StackNumberRevealed { cards, .. } => {
                for (index, card) in cards.into_iter().enumerate() {
                    let card_pose = uno_discard_pose(card);
                    let target = discard_position
                        + Vec2::new(
                            card_pose.0 - discard_top_pose.0,
                            card_pose.1 - discard_top_pose.1,
                        );
                    spawn_uno_flying_card(
                        &mut commands,
                        layer,
                        uno_card_handle(&assets, card),
                        Some(card),
                        draw_position,
                        target,
                        UNO_PLAY_CARD_DURATION + 0.08 + index as f32 * 0.12,
                        true,
                        index,
                        card_pose.2,
                    );
                }
            }
            UnoEvent::CardsDiscarded { player, cards } => {
                let source =
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                for (index, card) in cards.into_iter().take(8).enumerate() {
                    let pose = uno_discard_pose(card);
                    spawn_uno_flying_card(
                        &mut commands,
                        layer,
                        uno_card_handle(&assets, card),
                        Some(card),
                        source,
                        discard_position + Vec2::new(pose.0, pose.1),
                        UNO_PLAY_CARD_DURATION * 0.35 + index as f32 * 0.045,
                        false,
                        index,
                        pose.2,
                    );
                }
            }
            UnoEvent::ColorRouletteResolved {
                player,
                color,
                count,
            } => {
                spawn_uno_palette_effect(
                    &mut commands,
                    layer,
                    discard_position,
                    color,
                    &mut palette_materials,
                );
                spawn_uno_draw_cards_with_interval(
                    &mut commands,
                    layer,
                    draw_position,
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position),
                    count,
                    0.22,
                    0.18,
                    &assets,
                );
            }
            UnoEvent::DrawPenaltyReflected { target, count, .. } => {
                spawn_uno_draw_cards(
                    &mut commands,
                    layer,
                    draw_position,
                    uno_player_anchor_in_layer(target, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position),
                    count,
                    UNO_PLAY_CARD_DURATION * 0.72,
                    &assets,
                );
            }
            UnoEvent::ChallengeResolved {
                penalized, count, ..
            } => {
                spawn_uno_draw_cards(
                    &mut commands,
                    layer,
                    draw_position,
                    uno_player_anchor_in_layer(penalized, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position),
                    count,
                    0.18,
                    &assets,
                );
            }
            UnoEvent::SkipResolved {
                player,
                drew_card: true,
                ..
            } => {
                spawn_uno_draw_cards(
                    &mut commands,
                    layer,
                    draw_position,
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position),
                    1,
                    0.02,
                    &assets,
                );
            }
            UnoEvent::ColorChosen { color, .. } => {
                spawn_uno_palette_effect(
                    &mut commands,
                    layer,
                    discard_position,
                    color,
                    &mut palette_materials,
                );
            }
            UnoEvent::HandRefreshed { player, count } => {
                let player_position =
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                let visible = count.min(6);
                spawn_uno_transfer_cards(
                    &mut commands,
                    layer,
                    player_position,
                    discard_position,
                    visible,
                    UNO_PLAY_CARD_DURATION * 0.72,
                    &assets,
                );
                spawn_uno_draw_cards(
                    &mut commands,
                    layer,
                    draw_position,
                    player_position,
                    visible,
                    UNO_PLAY_CARD_DURATION * 0.72 + 0.34,
                    &assets,
                );
            }
            UnoEvent::SwapOneCardTaken { player, target } => {
                let source =
                    uno_player_anchor_in_layer(target, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                let target =
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                spawn_uno_transfer_cards(&mut commands, layer, source, target, 1, 0.0, &assets);
            }
            UnoEvent::SwapOneCompleted { player, target } => {
                let source =
                    uno_player_anchor_in_layer(player, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                let target =
                    uno_player_anchor_in_layer(target, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                spawn_uno_transfer_cards(&mut commands, layer, source, target, 1, 0.0, &assets);
            }
            UnoEvent::HandsTraded { first, second, .. } => {
                let first_position =
                    uno_player_anchor_in_layer(first, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                let second_position =
                    uno_player_anchor_in_layer(second, layer_node, layer_transform, &players)
                        .unwrap_or(discard_position);
                spawn_uno_transfer_cards(
                    &mut commands,
                    layer,
                    first_position,
                    second_position,
                    3,
                    0.0,
                    &assets,
                );
                spawn_uno_transfer_cards(
                    &mut commands,
                    layer,
                    second_position,
                    first_position,
                    3,
                    0.08,
                    &assets,
                );
            }
            UnoEvent::HandsPassed { direction, .. } => {
                let mut ring = game
                    .players
                    .iter()
                    .filter(|player| !player.eliminated)
                    .collect::<Vec<_>>();
                ring.sort_by_key(|player| player.seat.0);
                let count = ring.len();
                if count < 2 {
                    continue;
                }
                for (index, player) in ring.iter().enumerate() {
                    let target_index = match direction {
                        UnoDirection::Clockwise => (index + 1) % count,
                        UnoDirection::CounterClockwise => (index + count - 1) % count,
                    };
                    let source = uno_player_anchor_in_layer(
                        player.id,
                        layer_node,
                        layer_transform,
                        &players,
                    )
                    .unwrap_or(discard_position);
                    let target = uno_player_anchor_in_layer(
                        ring[target_index].id,
                        layer_node,
                        layer_transform,
                        &players,
                    )
                    .unwrap_or(discard_position);
                    spawn_uno_transfer_cards(
                        &mut commands,
                        layer,
                        source,
                        target,
                        2,
                        UNO_PLAY_CARD_DURATION * 0.72 + index as f32 * 0.035,
                        &assets,
                    );
                }
            }
            UnoEvent::Flipped { side } => {
                spawn_uno_flip_effect(
                    &mut commands,
                    layer,
                    layer_node.size() * layer_node.inverse_scale_factor(),
                    game,
                    side,
                    draw_position,
                    discard_position,
                    layer_node,
                    layer_transform,
                    &players,
                    &assets,
                );
            }
            UnoEvent::UnoCalled { .. }
            | UnoEvent::UnoReported { .. }
            | UnoEvent::SkipResolved {
                drew_card: false, ..
            }
            | UnoEvent::GameFinished { .. } => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_uno_flip_effect(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    game: &UnoSnapshot,
    side: leocard_uno::FlipSide,
    draw_position: Vec2,
    discard_position: Vec2,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    players: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    assets: &UiAssets,
) {
    let overlay = commands
        .spawn((
            UnoFlipOverlay { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(layer_size.x),
                height: px(layer_size.y),
                ..default()
            },
            BackgroundColor(match side {
                leocard_uno::FlipSide::Light => Color::srgba(1.0, 0.91, 0.53, 0.0),
                leocard_uno::FlipSide::Dark => Color::srgba(0.20, 0.08, 0.48, 0.0),
            }),
            GlobalZIndex(1490),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(overlay);

    let title = spawn_node(
        commands,
        overlay,
        Node {
            position_type: PositionType::Absolute,
            left: px(layer_size.x * 0.5 - 150.0),
            top: px(layer_size.y * 0.5 - 38.0),
            width: px(300),
            height: px(76),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(38)),
            ..default()
        },
        Some(Color::BLACK.with_alpha(0.68)),
    );
    commands.entity(title).insert((
        Outline::new(px(2.0), px(1.0), Color::WHITE.with_alpha(0.52)),
        BoxShadow::new(Color::BLACK.with_alpha(0.58), px(2), px(5), px(0), px(10)),
        GlobalZIndex(1492),
        FocusPolicy::Pass,
    ));
    add_text(
        commands,
        title,
        match side {
            leocard_uno::FlipSide::Light => "翻至亮面",
            leocard_uno::FlipSide::Dark => "翻至暗面",
        },
        27.0,
        Color::WHITE,
        assets,
    );

    let mut cards = Vec::new();
    for player in &game.players {
        let Some(position) =
            uno_player_anchor_in_layer(player.id, layer_node, layer_transform, players)
        else {
            continue;
        };
        if player.id == game.you {
            for (index, card) in game.your_hand.iter().copied().take(5).enumerate() {
                cards.push((
                    position + Vec2::new((index as f32 - 2.0) * 18.0, 18.0),
                    card.opposite_public_face()
                        .map(|old| uno_card_handle(assets, old))
                        .unwrap_or_else(|| assets.uno_card_back.clone()),
                    uno_card_handle(assets, card.public_face()),
                    index as f32 * 0.035,
                ));
            }
        } else if let Some(old) = player.inactive_hand.first().copied() {
            cards.push((
                position + Vec2::new(0.0, 24.0),
                uno_card_handle(assets, old),
                assets.uno_card_back.clone(),
                player.seat.0 as f32 * 0.035,
            ));
        }
    }
    if let Some(old) = game.draw_pile_inactive_top {
        cards.push((
            draw_position,
            uno_card_handle(assets, old),
            uno_card_handle(assets, old),
            0.08,
        ));
    }
    cards.push((
        discard_position,
        assets.uno_card_back.clone(),
        uno_card_handle(assets, game.discard_top),
        0.12,
    ));

    for (position, old_face, new_face, delay) in cards {
        let card = commands
            .spawn((
                UnoFlipCard {
                    elapsed: 0.0,
                    delay,
                    old_face: old_face.clone(),
                    new_face,
                    swapped: false,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(position.x - 34.0),
                    top: px(position.y - 53.0),
                    width: px(68),
                    height: px(106),
                    ..default()
                },
                ImageNode::new(old_face),
                UiTransform::default(),
                BoxShadow::new(Color::BLACK.with_alpha(0.54), px(3), px(6), px(0), px(8)),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(overlay).add_child(card);
    }
}

pub(in crate::app) fn uno_anchor_in_layer(
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
) -> Option<Vec2> {
    if node.size().min_element() <= 1.0 || layer_node.size().min_element() <= 1.0 {
        return None;
    }
    let inverse = layer_transform.try_inverse()?;
    Some(
        (inverse.transform_point2(transform.to_scale_angle_translation().2)
            + layer_node.size() * 0.5)
            * layer_node.inverse_scale_factor(),
    )
}

fn uno_player_anchor_in_layer(
    player: PlayerId,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    anchors: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
) -> Option<Vec2> {
    let (_, node, transform) = anchors.iter().find(|(anchor, _, _)| anchor.0 == player)?;
    uno_anchor_in_layer(node, transform, layer_node, layer_transform)
}

#[allow(clippy::too_many_arguments)]
fn spawn_uno_flying_card(
    commands: &mut Commands,
    layer: Entity,
    image: Handle<Image>,
    played_card: Option<UnoCard>,
    source: Vec2,
    target: Vec2,
    delay: f32,
    draw_animation: bool,
    index: usize,
    target_angle: f32,
) {
    let fan = (index as f32 % 7.0) - 3.0;
    let staging = source + Vec2::new(fan * 7.0, -32.0 - fan.abs() * 2.0);
    let midpoint = (source + target) * 0.5;
    let control = midpoint + Vec2::new(fan * 10.0, -95.0);
    let card = commands
        .spawn((
            UnoFlyingCard {
                elapsed: 0.0,
                delay,
                start: source,
                staging,
                control,
                target,
                duration: if draw_animation {
                    0.78
                } else {
                    UNO_PLAY_CARD_DURATION
                },
                draw_animation,
                played_card,
                start_angle: fan * 2.8,
                end_angle: if draw_animation {
                    fan * -1.4
                } else {
                    target_angle
                },
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(source.x - UNO_FLYING_CARD_WIDTH * 0.5),
                top: px(source.y - UNO_FLYING_CARD_HEIGHT * 0.5),
                width: px(UNO_FLYING_CARD_WIDTH),
                height: px(UNO_FLYING_CARD_HEIGHT),
                ..default()
            },
            ImageNode::new(image).with_color(Color::WHITE.with_alpha(0.0)),
            UiTransform::from_scale(Vec2::splat(0.76)),
            BoxShadow::new(Color::BLACK.with_alpha(0.42), px(2), px(5), px(0), px(6)),
            GlobalZIndex(1450),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(card);
}

fn spawn_uno_draw_cards(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    base_delay: f32,
    assets: &UiAssets,
) {
    spawn_uno_draw_cards_with_interval(
        commands, layer, source, target, count, base_delay, 0.045, assets,
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_uno_draw_cards_with_interval(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    base_delay: f32,
    interval: f32,
    assets: &UiAssets,
) {
    for index in 0..usize::from(count.min(16)) {
        spawn_uno_flying_card(
            commands,
            layer,
            assets.uno_card_back.clone(),
            None,
            source,
            target,
            base_delay + index as f32 * interval,
            true,
            index,
            0.0,
        );
    }
}

fn spawn_uno_transfer_cards(
    commands: &mut Commands,
    layer: Entity,
    source: Vec2,
    target: Vec2,
    count: u16,
    base_delay: f32,
    assets: &UiAssets,
) {
    for index in 0..usize::from(count.min(6)) {
        spawn_uno_flying_card(
            commands,
            layer,
            assets.uno_card_back.clone(),
            None,
            source,
            target,
            base_delay + index as f32 * 0.045,
            true,
            index,
            0.0,
        );
    }
}

/// 权威快照会在出牌动画结束前把新牌放进弃牌堆。动画等待布局或正在飞行时，
/// 暂时隐藏对应实体牌；双牌会同时隐藏，落地后再一起恢复。
pub(in crate::app) fn sync_uno_discard_reveal(
    client: Option<Res<ClientResource>>,
    presentation: Res<UnoPresentationState>,
    flights: Query<&UnoFlyingCard>,
    mut discards: Query<(&UnoDiscardCard, &mut Visibility)>,
) {
    let has_game = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
        .is_some();
    let active_cards = flights
        .iter()
        .filter_map(|flight| flight.played_card)
        .collect::<Vec<_>>();
    for (discard, mut visibility) in &mut discards {
        let hidden = has_game
            && uno_discard_should_be_hidden(
                discard.0,
                &presentation,
                active_cards.iter().copied().map(Some),
            );
        let expected = if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
}

pub(in crate::app) fn uno_discard_should_be_hidden(
    top: UnoCard,
    presentation: &UnoPresentationState,
    mut active_cards: impl Iterator<Item = Option<UnoCard>>,
) -> bool {
    presentation
        .events
        .iter()
        .any(|event| matches!(event, UnoEvent::CardPlayed { card, .. } if *card == top))
        || active_cards.any(|card| card == Some(top))
}

/// 弃牌堆只同步末尾六张牌；第七张加入时窗口会整体向前滑动，因此不能用数组
/// 下标决定姿态。物理牌标识在整局内稳定，用它分配偏移可让仍在堆中的旧牌原地不动。
pub(in crate::app) fn uno_discard_pose(card: UnoCard) -> (f32, f32, f32) {
    let color = match card.color() {
        Some(UnoColor::Red) => 0usize,
        Some(UnoColor::Yellow) => 1,
        Some(UnoColor::Green) => 2,
        Some(UnoColor::Blue) => 3,
        Some(UnoColor::Pink) => 4,
        Some(UnoColor::Teal) => 5,
        Some(UnoColor::Orange) => 6,
        Some(UnoColor::Purple) => 7,
        None => 8,
    };
    let face = match card.face() {
        UnoFace::Number(value) => usize::from(value),
        UnoFace::DrawTwo => 10,
        UnoFace::Reverse => 11,
        UnoFace::Skip => 12,
        UnoFace::Wild => 13,
        UnoFace::WildDrawFour => 14,
        UnoFace::SwapOne => 15,
        UnoFace::RefreshHand => 16,
        UnoFace::WildForceTrade => 17,
        UnoFace::WildPassHands => 18,
        UnoFace::ReverseDrawTwo => 19,
        UnoFace::ReverseSkip => 20,
        UnoFace::WildPowerReverse => 21,
        UnoFace::WildNoU => 22,
        UnoFace::StackOne => 23,
        UnoFace::StackTwo => 24,
        UnoFace::WildStackThree => 25,
        UnoFace::WildStackNumber => 26,
        UnoFace::DrawFour => 27,
        UnoFace::SkipEveryone => 28,
        UnoFace::DiscardAll => 29,
        UnoFace::WildReverseDrawFour => 30,
        UnoFace::WildDrawSix => 31,
        UnoFace::WildDrawTen => 32,
        UnoFace::WildColorRoulette => 33,
        UnoFace::DrawOne => 34,
        UnoFace::DrawFive => 35,
        UnoFace::Flip => 36,
        UnoFace::WildDrawTwo => 37,
        UnoFace::WildDrawColor => 38,
    };
    let key = color * 47 + face * 19 + usize::from(card.copy()) * 31;
    UNO_DISCARD_OFFSETS[(key ^ (key >> 3)) % UNO_DISCARD_OFFSETS.len()]
}

fn spawn_uno_palette_effect(
    commands: &mut Commands,
    layer: Entity,
    center: Vec2,
    selected: UnoColor,
    materials: &mut Assets<UnoPaletteMaterial>,
) {
    let base_material = materials.add(UnoPaletteMaterial::new(selected, false));
    let palette = commands
        .spawn((
            UnoPaletteEffect { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(center.x - 120.0),
                top: px(center.y - 120.0),
                width: px(240),
                height: px(240),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            MaterialNode(base_material),
            UiTransform::from_scale(Vec2::splat(0.78)),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(3), px(6), px(0), px(9)),
            GlobalZIndex(1500),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(palette);
    let sector_material = materials.add(UnoPaletteMaterial::new(selected, true));
    let selected_sector = commands
        .spawn((
            UnoPaletteSelectedSector { elapsed: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(240),
                height: px(240),
                ..default()
            },
            MaterialNode(sector_material),
            UiTransform::from_scale(Vec2::ONE),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(palette).add_child(selected_sector);

    let selected_color = uno_ui_color(selected);
    for (diameter, thickness, delay, start_scale, end_scale, max_alpha) in [
        // 两圈从调色盘外缘之外发射，避免涟漪起步时压在四色扇形上。
        (224.0, 6.0, 0.0, 1.16, 2.30, 0.82),
        (208.0, 2.5, 0.07, 1.19, 2.25, 0.68),
    ] {
        let ring = commands
            .spawn((
                UnoPaletteColorRing {
                    elapsed: 0.0,
                    delay,
                    color: selected_color,
                    start_scale,
                    end_scale,
                    max_alpha,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px((240.0 - diameter) * 0.5),
                    top: px((240.0 - diameter) * 0.5),
                    width: px(diameter),
                    height: px(diameter),
                    border: UiRect::all(px(thickness)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                BorderColor::all(selected_color.with_alpha(0.0)),
                UiTransform::from_scale(Vec2::splat(start_scale)),
                FocusPolicy::Pass,
                ZIndex(2),
            ))
            .id();
        commands.entity(palette).add_child(ring);
    }

    let sector_angle = match selected {
        UnoColor::Red => std::f32::consts::FRAC_PI_4,
        UnoColor::Yellow => std::f32::consts::FRAC_PI_4 * 3.0,
        UnoColor::Green => std::f32::consts::FRAC_PI_4 * 5.0,
        UnoColor::Blue => std::f32::consts::FRAC_PI_4 * 7.0,
        UnoColor::Pink => std::f32::consts::FRAC_PI_4,
        UnoColor::Teal => std::f32::consts::FRAC_PI_4 * 3.0,
        UnoColor::Orange => std::f32::consts::FRAC_PI_4 * 5.0,
        UnoColor::Purple => std::f32::consts::FRAC_PI_4 * 7.0,
    };
    for index in 0..10 {
        let spread = (index as f32 - 4.5) * 0.18;
        let angle = sector_angle + spread;
        let distance = 76.0 + (index % 4) as f32 * 12.0;
        let direction = Vec2::new(angle.cos(), angle.sin()) * distance;
        let size = 5.0 + (index % 3) as f32 * 1.7;
        let particle = commands
            .spawn((
                UnoPaletteParticle {
                    elapsed: 0.0,
                    delay: 0.72 + index as f32 * 0.018,
                    origin: Vec2::splat(120.0),
                    direction,
                    size: Vec2::splat(size),
                    color: selected_color,
                    rotation: index as f32 * 23.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(120.0 - size * 0.5),
                    top: px(120.0 - size * 0.5),
                    width: px(size),
                    height: px(size),
                    border_radius: BorderRadius::all(px(1.5)),
                    ..default()
                },
                BackgroundColor(selected_color.with_alpha(0.0)),
                UiTransform::from_scale(Vec2::splat(0.2)),
                FocusPolicy::Pass,
                ZIndex(3),
            ))
            .id();
        commands.entity(palette).add_child(particle);
    }
}

fn spawn_uno_reverse_effect(
    commands: &mut Commands,
    layer: Entity,
    layer_node: &ComputedNode,
    layer_transform: &UiGlobalTransform,
    game: &UnoSnapshot,
    anchors: &Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    effect_color: Color,
) {
    let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
    let own_effect_anchor = uno_reverse_own_anchor(layer_size);
    let mut ring = game
        .players
        .iter()
        .filter(|player| !player.eliminated)
        .filter_map(|player| {
            let position = if player.id == game.you {
                own_effect_anchor
            } else {
                uno_player_anchor_in_layer(player.id, layer_node, layer_transform, anchors)?
            };
            Some((player.id, position))
        })
        .collect::<Vec<_>>();
    let center = layer_size * 0.5;
    ring.sort_by(|(_, left), (_, right)| {
        let left_angle = (left.y - center.y).atan2(left.x - center.x);
        let right_angle = (right.y - center.y).atan2(right.x - center.x);
        left_angle.total_cmp(&right_angle)
    });
    if matches!(game.direction, UnoDirection::CounterClockwise) {
        ring.reverse();
    }
    if ring.len() < 2 {
        return;
    }
    let pair_count = if ring.len() == 2 { 1 } else { ring.len() };
    for index in 0..pair_count {
        let start = ring[index].1;
        let end = ring[(index + 1) % ring.len()].1;
        let midpoint = (start + end) * 0.5;
        let control = center + (midpoint - center) * 1.28;
        const SEGMENTS: usize = 18;
        for step in 0..SEGMENTS {
            let t0 = 0.13 + step as f32 / SEGMENTS as f32 * 0.64;
            let t1 = 0.13 + (step + 1) as f32 / SEGMENTS as f32 * 0.64;
            let start_point = quadratic_bezier(start, control, end, t0);
            let end_point = quadratic_bezier(start, control, end, t1);
            let direction = end_point - start_point;
            let t = (t0 + t1) * 0.5;
            let thickness = 7.0 + ((t - 0.13) / 0.64).clamp(0.0, 1.0) * 5.0;
            spawn_uno_reverse_arrow_part(
                commands,
                layer,
                (start_point + end_point) * 0.5,
                direction.y.atan2(direction.x).to_degrees(),
                index as f32 * 0.035 + step as f32 * 0.012,
                Vec2::new(direction.length() + 7.0, thickness),
                effect_color,
            );
        }
        let t = 0.84;
        let tip = quadratic_bezier(start, control, end, t);
        let tangent = (control - start) * (2.0 * (1.0 - t)) + (end - control) * (2.0 * t);
        let direction = tangent.normalize_or(Vec2::X);
        let side = Vec2::new(-direction.y, direction.x);
        let angle = direction.y.atan2(direction.x).to_degrees();
        let back = tip - direction * 16.0;
        let head_delay = index as f32 * 0.035 + 0.24;
        spawn_uno_reverse_arrow_part(
            commands,
            layer,
            back - side * 9.0,
            angle + 34.0,
            head_delay,
            Vec2::new(40.0, 13.0),
            effect_color,
        );
        spawn_uno_reverse_arrow_part(
            commands,
            layer,
            back + side * 9.0,
            angle - 34.0,
            head_delay,
            Vec2::new(40.0, 13.0),
            effect_color,
        );
    }
}

/// 反转特效把自己视作下方居中的操作区座位，而不是左下角的信息框。
pub(in crate::app) fn uno_reverse_own_anchor(layer_size: Vec2) -> Vec2 {
    Vec2::new(
        layer_size.x * 0.5,
        layer_size.y - UNO_ACTION_AREA_BOTTOM - UNO_ACTION_AREA_HEIGHT * 0.5,
    )
}

fn spawn_uno_reverse_arrow_part(
    commands: &mut Commands,
    layer: Entity,
    position: Vec2,
    angle: f32,
    delay: f32,
    size: Vec2,
    effect_color: Color,
) {
    let radians = angle.to_radians();
    let normal = Vec2::new(-radians.sin(), radians.cos());
    for (offset, layer_size, color, max_alpha, shadow_alpha, z_index, extra_delay) in [
        (
            Vec2::new(2.5, 4.0),
            Vec2::new(size.x + 5.0, size.y + 6.0),
            Color::BLACK,
            0.38,
            0.24,
            1489,
            0.0,
        ),
        (
            Vec2::new(1.2, 2.6),
            Vec2::new(size.x + 1.0, size.y + 1.5),
            effect_color.mix(&Color::BLACK, 0.46),
            0.94,
            0.18,
            1490,
            0.010,
        ),
        (Vec2::ZERO, size, effect_color, 0.98, 0.20, 1491, 0.022),
        (
            -normal * (size.y * 0.30),
            Vec2::new((size.x - 5.0).max(4.0), (size.y * 0.18).max(1.8)),
            effect_color.mix(&Color::WHITE, 0.74),
            0.82,
            0.06,
            1492,
            0.060,
        ),
    ] {
        spawn_uno_reverse_arrow_layer(
            commands,
            layer,
            position + offset,
            angle,
            delay + extra_delay,
            layer_size,
            color,
            max_alpha,
            shadow_alpha,
            z_index,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_uno_reverse_arrow_layer(
    commands: &mut Commands,
    layer: Entity,
    position: Vec2,
    angle: f32,
    delay: f32,
    size: Vec2,
    color: Color,
    max_alpha: f32,
    shadow_alpha: f32,
    z_index: i32,
) {
    let arrow = commands
        .spawn((
            UnoReverseArrow {
                elapsed: 0.0,
                delay,
                color,
                max_alpha,
                shadow_alpha,
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x - size.x * 0.5),
                top: px(position.y - size.y * 0.5),
                width: px(size.x),
                height: px(size.y),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(color.with_alpha(0.0)),
            UiTransform::from_rotation(Rot2::degrees(angle)),
            BoxShadow::new(Color::BLACK.with_alpha(0.0), px(1), px(2), px(0), px(4)),
            GlobalZIndex(z_index),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(arrow);
}

fn quadratic_bezier(start: Vec2, control: Vec2, end: Vec2, t: f32) -> Vec2 {
    start * (1.0 - t).powi(2) + control * (2.0 * (1.0 - t) * t) + end * t.powi(2)
}

pub(in crate::app) fn animate_uno_flying_cards(
    time: Res<Time>,
    mut commands: Commands,
    mut cards: Query<(
        Entity,
        &mut UnoFlyingCard,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
    )>,
) {
    for (entity, mut flight, mut node, mut transform, mut image) in &mut cards {
        flight.elapsed += time.delta_secs();
        let local = flight.elapsed - flight.delay;
        if local < 0.0 {
            continue;
        }
        let progress = (local / flight.duration).clamp(0.0, 1.0);
        let (position, motion) = if flight.draw_animation {
            if progress < 0.22 {
                let t = smoothstep(progress / 0.22);
                (flight.start.lerp(flight.staging, t), t * 0.12)
            } else if progress < 0.43 {
                (flight.staging, 0.12)
            } else {
                let t = smoothstep((progress - 0.43) / 0.57);
                (
                    quadratic_bezier(flight.staging, flight.control, flight.target, t),
                    0.12 + t * 0.88,
                )
            }
        } else {
            let t = 1.0 - (1.0 - progress).powi(3);
            (
                quadratic_bezier(flight.start, flight.control, flight.target, t),
                t,
            )
        };
        node.left = px(position.x - UNO_FLYING_CARD_WIDTH * 0.5);
        node.top = px(position.y - UNO_FLYING_CARD_HEIGHT * 0.5);
        transform.rotation =
            Rot2::degrees(flight.start_angle + (flight.end_angle - flight.start_angle) * motion);
        transform.scale = Vec2::splat(uno_flying_card_scale(flight.draw_animation, progress));
        let fade_in = (local / 0.08).clamp(0.0, 1.0);
        let fade_out = if flight.draw_animation {
            ((1.0 - progress) / 0.10).clamp(0.0, 1.0)
        } else {
            1.0
        };
        image.color = Color::WHITE.with_alpha(fade_in * fade_out);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(in crate::app) fn animate_uno_flip_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut overlays: Query<(Entity, &mut UnoFlipOverlay, &mut BackgroundColor)>,
    mut cards: Query<(&mut UnoFlipCard, &mut UiTransform, &mut ImageNode)>,
) {
    let delta = time.delta_secs();
    for (entity, mut effect, mut background) in &mut overlays {
        effect.elapsed += delta;
        let progress = (effect.elapsed / 1.55).clamp(0.0, 1.0);
        let alpha = (std::f32::consts::PI * progress).sin().powf(1.35) * 0.34;
        background.0.set_alpha(alpha);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (mut card, mut transform, mut image) in &mut cards {
        card.elapsed += delta;
        let progress = ((card.elapsed - card.delay) / 1.05).clamp(0.0, 1.0);
        if progress <= 0.0 {
            transform.scale = Vec2::splat(0.86);
            continue;
        }
        if progress >= 0.5 && !card.swapped {
            image.image = card.new_face.clone();
            card.swapped = true;
        } else if progress < 0.5 && card.swapped {
            image.image = card.old_face.clone();
            card.swapped = false;
        }
        let edge = (std::f32::consts::PI * progress).cos().abs().max(0.035);
        let lift = (std::f32::consts::PI * progress).sin();
        transform.scale = Vec2::new(edge * (0.86 + lift * 0.20), 0.86 + lift * 0.15);
        transform.rotation = Rot2::degrees((progress - 0.5) * 9.0);
        transform.translation = Val2::px(0.0, -lift * 18.0);
    }
}

pub(in crate::app) fn uno_flying_card_scale(draw_animation: bool, progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if !draw_animation && progress >= 1.0 {
        return 1.0;
    }
    if draw_animation {
        0.76 + (progress * std::f32::consts::PI).sin() * 0.10
    } else {
        let settle = smoothstep(progress);
        0.76 + settle * 0.24 + (progress * std::f32::consts::PI).sin() * 0.08
    }
}

pub(in crate::app) fn animate_uno_palette_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut palettes: Query<(
        Entity,
        &mut UnoPaletteEffect,
        &mut UiTransform,
        &MaterialNode<UnoPaletteMaterial>,
        &mut BoxShadow,
    )>,
) {
    for (entity, mut effect, mut transform, material_node, mut shadow) in &mut palettes {
        effect.elapsed += time.delta_secs();
        let progress = (effect.elapsed / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let entry = ease_out_back((progress / 0.17).clamp(0.0, 1.0));
        let fade = smoothstep(((1.0 - progress) / 0.15).clamp(0.0, 1.0));
        transform.scale = Vec2::splat(0.72 + entry * 0.28);
        transform.rotation = Rot2::degrees((1.0 - entry) * -16.0);
        let opacity = entry.clamp(0.0, 1.0) * fade;
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = opacity;
        }
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(opacity * 0.50);
            style.blur_radius = px(9.0 + entry * 5.0);
        }
        if progress >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(in crate::app) fn animate_uno_palette_selected_sectors(
    time: Res<Time>,
    mut materials: ResMut<Assets<UnoPaletteMaterial>>,
    mut sectors: Query<(
        &mut UnoPaletteSelectedSector,
        &mut UiTransform,
        &MaterialNode<UnoPaletteMaterial>,
    )>,
) {
    for (mut effect, mut transform, material_node) in &mut sectors {
        effect.elapsed += time.delta_secs();
        let progress = (effect.elapsed / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let entry = smoothstep((progress / 0.16).clamp(0.0, 1.0));
        let fade = smoothstep(((1.0 - progress) / 0.15).clamp(0.0, 1.0));
        transform.scale = Vec2::splat(uno_palette_selected_scale(progress));
        if let Some(mut material) = materials.get_mut(&material_node.0) {
            material.params.z = entry * fade;
        }
    }
}

pub(in crate::app) fn animate_uno_palette_color_rings(
    time: Res<Time>,
    mut rings: Query<(&mut UnoPaletteColorRing, &mut BorderColor, &mut UiTransform)>,
) {
    for (mut ring, mut border, mut transform) in &mut rings {
        ring.elapsed += time.delta_secs();
        let progress = ((ring.elapsed - ring.delay) / UNO_PALETTE_EFFECT_DURATION).clamp(0.0, 1.0);
        let burst = ((progress - 0.34) / 0.43).clamp(0.0, 1.0);
        let motion = smoothstep(burst);
        let opacity = (burst * std::f32::consts::PI).sin().max(0.0).powf(0.72);
        transform.scale =
            Vec2::splat(ring.start_scale + (ring.end_scale - ring.start_scale) * motion);
        border.set_all(ring.color.with_alpha(opacity * ring.max_alpha));
    }
}

pub(in crate::app) fn animate_uno_palette_particles(
    time: Res<Time>,
    mut particles: Query<(
        &mut UnoPaletteParticle,
        &mut Node,
        &mut BackgroundColor,
        &mut UiTransform,
    )>,
) {
    for (mut particle, mut node, mut background, mut transform) in &mut particles {
        particle.elapsed += time.delta_secs();
        let local = ((particle.elapsed - particle.delay) / 0.82).clamp(0.0, 1.0);
        let motion = ease_out_cubic(local);
        let opacity = (local * std::f32::consts::PI).sin().max(0.0).powf(0.65);
        let position = particle.origin + particle.direction * motion;
        node.left = px(position.x - particle.size.x * 0.5);
        node.top = px(position.y - particle.size.y * 0.5);
        background.0 = particle.color.with_alpha(opacity * 0.92);
        transform.scale = Vec2::splat(0.25 + opacity * 0.95);
        transform.rotation = Rot2::degrees(particle.rotation + motion * 115.0);
    }
}

pub(in crate::app) fn uno_palette_selected_scale(progress: f32) -> f32 {
    let progress = progress.clamp(0.0, 1.0);
    if progress <= 0.18 {
        1.0
    } else if progress <= 0.62 {
        1.0 + ease_out_back((progress - 0.18) / 0.44) * 0.32
    } else {
        1.32 - smoothstep((progress - 0.62) / 0.38) * 0.07
    }
}

pub(in crate::app) fn animate_uno_reverse_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut arrows: Query<(
        Entity,
        &mut UnoReverseArrow,
        &mut BackgroundColor,
        &mut BoxShadow,
        &mut UiTransform,
    )>,
) {
    for (entity, mut arrow, mut background, mut shadow, mut transform) in &mut arrows {
        arrow.elapsed += time.delta_secs();
        let local = arrow.elapsed - arrow.delay;
        if local < 0.0 {
            continue;
        }
        let entry = ease_out_back((local / 0.16).clamp(0.0, 1.0));
        let fade =
            smoothstep(((UNO_REVERSE_EFFECT_DURATION - arrow.elapsed) / 0.30).clamp(0.0, 1.0));
        let opacity = entry.clamp(0.0, 1.0) * fade;
        background.0 = arrow.color.with_alpha(opacity * arrow.max_alpha);
        if let Some(style) = shadow.0.first_mut() {
            style.color = Color::BLACK.with_alpha(opacity * arrow.shadow_alpha);
        }
        transform.scale = Vec2::splat(0.68 + entry * 0.32);
        if arrow.elapsed >= UNO_REVERSE_EFFECT_DURATION {
            commands.entity(entity).despawn();
        }
    }
}

fn ease_out_back(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    let shifted = value - 1.0;
    1.0 + 2.70158 * shifted.powi(3) + 1.70158 * shifted.powi(2)
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}
