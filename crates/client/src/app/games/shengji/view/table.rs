use super::super::{
    ShengjiAssets, ShengjiBottomFlipPanelElement, ShengjiDealerBadge, ShengjiLevelIndicator,
    ShengjiPresentationState, ShengjiScoreCaptureEffectState, ShengjiSettlementAnimation,
    ShengjiUiAction, ShengjiUiState, add_shengji_presentation_overlay,
};
use super::{
    ShengjiCardSize, add_shengji_actions, add_shengji_bidding_panel, add_shengji_card_row,
    add_shengji_collecting_tray, add_shengji_hand, add_shengji_own_play, add_shengji_play_area,
    add_shengji_result, add_shengji_self_panel, add_shengji_throw_penalty_effect,
    select_forced_shengji_follow_cards, shengji_display_trump, shengji_level_label,
};
use crate::app::presentation::{
    ACCENT, BORDER, HEADER_BG, MUTED, PanelSkin, PlayerMenuProfile, StartGameSeatTransition, TEXT,
    TableBackground, TableBackgroundMaterial, TurnBorderAnimationKey, TurnBorderMaterial,
    add_auto_play_overlay, add_auto_play_robot_indicator, add_avatar, add_interaction_menu,
    add_text, add_turn_border_trace, attach_start_game_seat_transition, decorate_panel_skin,
    decorate_player_panel, spawn_node, table_material_params,
};
use crate::app::runtime::{AvatarImages, ClientResource, TableAppearance, UiAssets};
use crate::app::shell::{
    ChatAuxiliaryAction, ChatPanelState, OpponentBadge, PlayerAvatarAnchor, SeatSide,
    SocialUiAction, SocialUiState, UiAction, add_chat_panel, add_reconnecting_overlay,
};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_client::NetworkState;
use leocard_protocol::ShengjiBottomFlipRevealView;
use leocard_protocol::{
    GameKind, PlayerId, SeatId, ShengjiPhaseView, ShengjiPlayerState, ShengjiPublicPlay,
    ShengjiSnapshot,
};

const SHENGJI_PLAYER_COUNT: u8 = 4;

pub(crate) struct ShengjiTableVisuals<'a> {
    pub assets: &'a UiAssets,
    pub game_assets: &'a ShengjiAssets,
    pub avatars: &'a AvatarImages,
    pub appearance: &'a TableAppearance,
    pub brightness: f32,
    pub vignette: f32,
    pub table_materials: &'a mut Assets<TableBackgroundMaterial>,
    pub turn_border_materials: &'a mut Assets<TurnBorderMaterial>,
    pub start_game_transition: &'a StartGameSeatTransition,
    pub score_capture: &'a ShengjiScoreCaptureEffectState,
    pub settlement: &'a ShengjiSettlementAnimation,
    pub presentation: &'a ShengjiPresentationState,
}

#[expect(
    clippy::too_many_arguments,
    reason = "the game-screen adapter passes common screen state plus grouped visuals"
)]
pub(crate) fn render_shengji_table(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    game: &ShengjiSnapshot,
    ui: &mut ShengjiUiState,
    social: &SocialUiState,
    chat: &ChatPanelState,
    visuals: ShengjiTableVisuals,
) {
    if ui.observed_hand.observe((game.match_id, game.hand_number)) {
        ui.selected.clear();
        ui.buried_open = false;
    }
    if game.your_buried.is_empty() {
        ui.buried_open = false;
    }
    ui.selected.retain(|card| game.your_hand.contains(card));
    select_forced_shengji_follow_cards(game, ui);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        None,
    );
    let felt = visuals
        .appearance
        .custom_felt
        .as_ref()
        .unwrap_or(&visuals.assets.table_felt)
        .clone();
    let material = visuals.table_materials.add(TableBackgroundMaterial {
        params: table_material_params(
            visuals.brightness,
            visuals.vignette,
            visuals.appearance.custom_felt.is_none(),
        ),
        texture: felt,
    });
    commands
        .entity(content)
        .insert((MaterialNode(material), TableBackground));

    let table = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(410),
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    if let NetworkState::Reconnecting(message) = client.0.state() {
        add_reconnecting_overlay(commands, table, message, visuals.assets);
    }

    let own = game
        .players
        .iter()
        .find(|player| player.id == game.you)
        .expect("双升快照包含接收者");
    let start_transition_active = visuals.start_game_transition.is_active_for(game.match_id);
    let previous_trick = visuals.presentation.revealed_previous_trick();
    for relative in 1..SHENGJI_PLAYER_COUNT {
        let seat = SeatId((own.seat.0 + relative) % SHENGJI_PLAYER_COUNT);
        if let Some(player) = game.players.iter().find(|player| player.seat == seat) {
            add_shengji_opponent(
                commands,
                table,
                relative,
                player,
                game,
                visuals.assets,
                visuals.avatars,
                social.interaction_menu_open,
                visuals.turn_border_materials,
                previous_trick,
                start_transition_active,
            );
        }
    }
    let bidding_visible = matches!(
        game.phase,
        ShengjiPhaseView::Dealing { .. } | ShengjiPhaseView::BiddingGrace { .. }
    );
    add_shengji_own_play(commands, table, game, visuals.assets, previous_trick);
    add_shengji_collecting_tray(
        commands,
        table,
        game,
        client.0.model().shengji_collected_score_cards(),
        visuals.score_capture,
        visuals.assets,
    );
    add_shengji_throw_penalty_effect(commands, table, game, visuals.assets);
    if let ShengjiPhaseView::BottomFlipping { reveal } = &game.phase {
        add_shengji_bottom_flip(commands, table, game, reveal.as_ref(), visuals.assets);
    }
    add_shengji_presentation_overlay(
        commands,
        table,
        game,
        visuals.presentation,
        visuals.assets,
        visuals.game_assets,
    );

    let finished = matches!(game.phase, ShengjiPhaseView::Finished { .. });
    if ui.buried_open && !finished {
        add_shengji_private_buried(commands, table, game, visuals.assets);
    }
    if !finished {
        let hand_area = spawn_node(
            commands,
            content,
            Node {
                width: percent(100),
                height: px(180),
                flex_shrink: 0.0,
                position_type: PositionType::Relative,
                ..default()
            },
            None,
        );
        if bidding_visible {
            add_shengji_bidding_panel(commands, hand_area, game, visuals.assets);
        }
        add_shengji_actions(commands, hand_area, game, ui, visuals.assets);
        add_shengji_hand(commands, hand_area, game, ui, visuals.assets);
        add_shengji_self_panel(
            commands,
            hand_area,
            own,
            game,
            visuals.assets,
            visuals.avatars,
            visuals.turn_border_materials,
            start_transition_active,
        );
    }
    let local_auto_play = matches!(
        game.phase,
        ShengjiPhaseView::Burying
            | ShengjiPhaseView::BottomCopying { .. }
            | ShengjiPhaseView::BottomCopyBurying { .. }
            | ShengjiPhaseView::FiveTrumpCrossing { .. }
            | ShengjiPhaseView::Playing
    )
    .then_some(own.auto_play);
    let previous_trick =
        shengji_previous_trick_button_state(&game.phase, visuals.presentation.has_previous_trick());
    let buried_cards = !game.your_buried.is_empty() && !finished;
    let auxiliary_actions = [
        ChatAuxiliaryAction {
            label: "上轮",
            action: previous_trick
                .filter(|available| *available)
                .map(|_| UiAction::Shengji(ShengjiUiAction::ShowPreviousTrick)),
            highlighted: false,
        },
        ChatAuxiliaryAction {
            label: "底牌",
            action: buried_cards.then_some(UiAction::Shengji(ShengjiUiAction::ToggleBuried)),
            highlighted: true,
        },
    ];
    add_chat_panel(
        commands,
        content,
        chat,
        visuals.assets,
        local_auto_play,
        &auxiliary_actions,
    );
    if local_auto_play == Some(true) {
        add_auto_play_overlay(commands, content, visuals.assets);
    }

    if let ShengjiPhaseView::Finished { result, buried } = &game.phase {
        add_shengji_result(
            commands,
            table,
            game,
            result,
            buried,
            visuals.assets,
            visuals.avatars,
            visuals.settlement,
        );
    }
}

pub(crate) fn shengji_previous_trick_button_state(
    phase: &ShengjiPhaseView,
    has_previous_trick: bool,
) -> Option<bool> {
    matches!(phase, ShengjiPhaseView::Playing).then_some(has_previous_trick)
}

fn add_shengji_private_buried(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
) {
    let backdrop = spawn_node(
        commands,
        table,
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
        Some(Color::BLACK.with_alpha(0.48)),
    );
    commands.entity(backdrop).insert((
        Button,
        UiAction::Shengji(ShengjiUiAction::ToggleBuried),
        GlobalZIndex(1750),
    ));

    let panel = spawn_node(
        commands,
        backdrop,
        Node {
            min_width: px(430),
            min_height: px(184),
            padding: UiRect::axes(px(28), px(22)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(11),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.98)),
    );
    commands.entity(panel).insert((
        BorderColor::all(BORDER),
        BoxShadow::new(Color::BLACK.with_alpha(0.55), px(2), px(8), px(0), px(15)),
        FocusPolicy::Pass,
    ));
    decorate_panel_skin(commands, panel, PanelSkin::Popup, assets);
    add_text(commands, panel, "我的底牌", 22.0, ACCENT, assets);
    add_shengji_card_row(
        commands,
        panel,
        &game.your_buried,
        ShengjiCardSize::Seat,
        shengji_display_trump(game),
        assets,
    );
    add_text(
        commands,
        panel,
        "仅你可见 · 点击任意位置收起",
        13.0,
        MUTED,
        assets,
    );
}

fn add_shengji_bottom_flip(
    commands: &mut Commands,
    table: Entity,
    game: &ShengjiSnapshot,
    reveal: Option<&ShengjiBottomFlipRevealView>,
    assets: &UiAssets,
) {
    let panel = spawn_node(
        commands,
        table,
        Node {
            position_type: PositionType::Absolute,
            left: percent(31),
            right: percent(31),
            top: percent(18),
            min_height: px(210),
            padding: UiRect::all(px(20)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(9),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.94)),
    );
    commands.entity(panel).insert((
        GlobalZIndex(900),
        FocusPolicy::Pass,
        BoxShadow::new(Color::BLACK.with_alpha(0.45), px(2), px(7), px(0), px(12)),
    ));
    decorate_panel_skin(commands, panel, PanelSkin::Section, assets);
    add_text(commands, panel, "扳底", 22.0, ACCENT, assets);

    let Some(reveal) = reveal else {
        add_text(commands, panel, "准备翻开底牌…", 15.0, MUTED, assets);
        return;
    };
    let central_card = add_shengji_card_row(
        commands,
        panel,
        &[reveal.card],
        ShengjiCardSize::Seat,
        shengji_display_trump(game),
        assets,
    );
    commands.entity(central_card).insert((
        ShengjiBottomFlipPanelElement::CentralCard,
        UiTransform::IDENTITY,
        Visibility::Visible,
    ));
    if reveal.matches.is_empty() {
        add_text(
            commands,
            panel,
            "无人持有同牌，继续翻牌",
            14.0,
            MUTED,
            assets,
        );
    } else {
        for (index, matched) in reveal.matches.iter().enumerate() {
            let row = spawn_node(
                commands,
                panel,
                Node {
                    min_height: px(52),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: px(10),
                    ..default()
                },
                None,
            );
            commands.entity(row).insert((
                ShengjiBottomFlipPanelElement::MatchRow {
                    player: matched.player,
                    index,
                    count: reveal.matches.len(),
                },
                UiTransform::IDENTITY,
                Visibility::Visible,
            ));
            let name = game
                .players
                .iter()
                .find(|player| player.id == matched.player)
                .map_or("玩家", |player| player.name.as_str());
            add_text(commands, row, name, 15.0, TEXT, assets);
            add_shengji_card_row(
                commands,
                row,
                &matched.cards,
                ShengjiCardSize::Score,
                shengji_display_trump(game),
                assets,
            );
        }
    }
    if let Some(dealer) = reveal.dealer {
        let name = game
            .players
            .iter()
            .find(|player| player.id == dealer)
            .map_or("该玩家", |player| player.name.as_str());
        let resolution = add_text(
            commands,
            panel,
            format!("{name} 坐庄"),
            18.0,
            ACCENT,
            assets,
        );
        commands.entity(resolution).insert((
            ShengjiBottomFlipPanelElement::DealerLine,
            UiTransform::IDENTITY,
            Visibility::Visible,
        ));
    } else {
        let resolution = add_text(commands, panel, "尚未定庄，继续翻牌", 14.0, MUTED, assets);
        commands.entity(resolution).insert((
            ShengjiBottomFlipPanelElement::DealerLine,
            UiTransform::IDENTITY,
            Visibility::Visible,
        ));
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the opponent builder keeps seat state and visual resources explicit"
)]
fn add_shengji_opponent(
    commands: &mut Commands,
    table: Entity,
    relative: u8,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    previous_trick: Option<&[ShengjiPublicPlay]>,
    start_transition_active: bool,
) {
    // 机器人图标会从人物框向牌桌内侧伸出 38px；左右出牌区额外留白，
    // 保证牌、机器人标记和人物框各自拥有清晰的视觉边界。
    const SIDE_PLAY_GAP: f32 = 68.0;
    const SIDE_SLOT_WIDTH: f32 = 188.0 + SIDE_PLAY_GAP + 165.0;
    let side = match relative {
        1 => SeatSide::Left,
        2 => SeatSide::Top,
        3 => SeatSide::Right,
        _ => unreachable!(),
    };
    let mut node = Node {
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        column_gap: px(if matches!(side, SeatSide::Top) {
            12.0
        } else {
            SIDE_PLAY_GAP
        }),
        row_gap: px(7),
        ..default()
    };
    match side {
        SeatSide::Left => {
            node.left = px(12);
            node.top = percent(50);
            node.width = px(SIDE_SLOT_WIDTH);
        }
        SeatSide::Top => {
            node.left = percent(32);
            node.right = percent(32);
            node.top = px(12);
            node.flex_direction = FlexDirection::Column;
        }
        SeatSide::Right => {
            node.right = px(12);
            node.top = percent(50);
            node.width = px(SIDE_SLOT_WIDTH);
            node.justify_content = JustifyContent::FlexEnd;
        }
    }
    let slot = spawn_node(commands, table, node, None);
    if !matches!(side, SeatSide::Top) {
        // 百分比定位以节点上沿为基准；回拉半个出牌槽高度，使左右人物框
        // 在任意窗口比例下都真正落在牌桌垂直中线上。
        commands
            .entity(slot)
            .insert(UiTransform::from_translation(Val2::px(0.0, -52.0)));
    }
    if matches!(side, SeatSide::Right) {
        add_shengji_play_area(
            commands,
            slot,
            game,
            player.id,
            side,
            assets,
            previous_trick,
        );
    }
    add_shengji_player_panel(
        commands,
        slot,
        player,
        game,
        side,
        assets,
        avatars,
        interaction_menu_open,
        turn_border_materials,
        start_transition_active,
    );
    if !matches!(side, SeatSide::Right) {
        add_shengji_play_area(
            commands,
            slot,
            game,
            player.id,
            side,
            assets,
            previous_trick,
        );
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "the player-panel builder keeps identity, score, and animation inputs explicit"
)]
fn add_shengji_player_panel(
    commands: &mut Commands,
    parent: Entity,
    player: &ShengjiPlayerState,
    game: &ShengjiSnapshot,
    side: SeatSide,
    assets: &UiAssets,
    avatars: &AvatarImages,
    interaction_menu_open: Option<PlayerId>,
    turn_border_materials: &mut Assets<TurnBorderMaterial>,
    start_transition_active: bool,
) {
    let current = game.current_player == Some(player.id);
    let panel = spawn_node(
        commands,
        parent,
        Node {
            width: px(188),
            min_width: px(188),
            height: px(72),
            min_height: px(72),
            // 双升人物框没有七鬼/德州的内侧大数值区，使用完全对称的
            // padding，确保“头像 + 文字”组合在三个方向上都处于框中心。
            padding: UiRect::axes(px(8), px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(8),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands.entity(panel).insert((
        BorderColor::all(BORDER),
        Button,
        UiAction::Social(SocialUiAction::ToggleInteractionMenu(player.id)),
    ));
    attach_start_game_seat_transition(commands, panel, player.id, start_transition_active);
    decorate_player_panel(commands, panel, assets, 1.0);
    if current {
        add_turn_border_trace(
            commands,
            panel,
            turn_border_materials,
            TurnBorderAnimationKey::new(GameKind::Shengji, game.match_id, player.id),
        );
    }
    let avatar = player.avatar.and_then(|id| avatars.remote.get(&id));
    let avatar_entity = add_avatar(commands, panel, &player.name, avatar, 34.0, assets);
    commands
        .entity(avatar_entity)
        .insert(PlayerAvatarAnchor(player.id));
    if player.auto_play {
        add_auto_play_robot_indicator(commands, panel, player.id, side, assets);
    }
    let details = spawn_node(
        commands,
        panel,
        Node {
            min_width: px(82),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        None,
    );
    add_text(commands, details, &player.name, 14.0, TEXT, assets);
    let level = add_text(
        commands,
        details,
        format!(
            "{} · {}张",
            shengji_level_label(game.levels[usize::from(player.id.0 % 2)]),
            player.hand_len
        ),
        11.0,
        MUTED,
        assets,
    );
    commands
        .entity(level)
        .insert(ShengjiLevelIndicator { base_color: MUTED });
    if game.dealer == Some(player.id) {
        let dealer = spawn_node(
            commands,
            panel,
            Node {
                position_type: PositionType::Absolute,
                right: if matches!(side, SeatSide::Right) {
                    Val::Auto
                } else {
                    px(5)
                },
                left: if matches!(side, SeatSide::Right) {
                    px(5)
                } else {
                    Val::Auto
                },
                top: px(4),
                width: px(24),
                height: px(24),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            Some(ACCENT),
        );
        commands.entity(dealer).insert(ShengjiDealerBadge);
        add_text(commands, dealer, "庄", 12.0, Color::BLACK, assets);
    }
    let interaction_menu = add_interaction_menu(
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
        .entity(interaction_menu)
        .insert(if interaction_menu_open == Some(player.id) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        });
    commands.entity(panel).insert(OpponentBadge {
        player: player.id,
        score_popup: None,
        interaction_menu,
    });
}
