//! Top-level connection, lobby, settings, and table screen composition.

use super::*;

pub(super) fn render_ui(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    form: Res<ConnectionForm>,
    profile: Res<LocalPlayerProfile>,
    visuals: VisualAssets,
    chat: Res<ChatPanelState>,
    developer_hand: Res<DeveloperHandInput>,
    mut table_materials: ResMut<Assets<TableBackgroundMaterial>>,
    mut turn_border_materials: ResMut<Assets<TurnBorderMaterial>>,
    mut ui: ResMut<UiState>,
    old_roots: Query<Entity, With<UiRoot>>,
) {
    if !ui.dirty {
        return;
    }
    ui.dirty = false;
    for entity in &old_roots {
        commands.entity(entity).despawn();
    }
    let in_uno_lobby = client
        .as_deref()
        .and_then(|client| client.0.model().lobby())
        .is_some_and(|lobby| lobby.game == GameKind::Uno);
    if !in_uno_lobby {
        ui.uno_expansion_settings_open = false;
    }

    if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().qigui523_game())
    {
        ui.selected.retain(|card| game.your_hand.contains(card));
        ui.card_animations
            .retain(|card, _| game.your_hand.contains(card));
        if ui
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().texas_holdem_game())
    {
        ui.selected.clear();
        ui.card_animations.clear();
        ui.observed_hand.clear();
        ui.greedy_hint.reset();
        if ui
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().shengji_game())
    {
        ui.selected.clear();
        ui.card_animations.clear();
        ui.observed_hand.clear();
        ui.greedy_hint.reset();
        ui.selected_shengji
            .retain(|card| game.your_hand.contains(card));
        if ui
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.interaction_menu_open = None;
        }
    } else if let Some(game) = client
        .as_deref()
        .and_then(|client| client.0.model().uno_game())
    {
        ui.selected.clear();
        ui.card_animations.clear();
        ui.observed_hand.clear();
        ui.selected_shengji.clear();
        ui.shengji_card_animations.clear();
        ui.observed_shengji_hand.clear();
        ui.greedy_hint.reset();
        ui.selected_uno.retain(|card| game.your_hand.contains(card));
        if let Some(card) = game.your_jump_in_card {
            ui.selected_uno.clear();
            ui.selected_uno.insert(card);
        } else if game.current_player != Some(game.you) {
            ui.selected_uno.clear();
        }
        ui.uno_card_animations
            .retain(|card, _| game.your_hand.contains(card));
        let selecting_swap_targets = matches!(
            game.pending_swap,
            Some(UnoPendingSwapView::SwapOneTarget { player })
                | Some(UnoPendingSwapView::ForceTrade { player })
                | Some(UnoPendingSwapView::SevenSwap { player }) if player == game.you
        );
        if !selecting_swap_targets {
            ui.uno_swap_targets.clear();
        } else {
            ui.interaction_menu_open = None;
            ui.uno_swap_targets.retain(|target| {
                game.players
                    .iter()
                    .any(|player| player.id == *target && !player.eliminated)
            });
        }
        if ui
            .uno_color_choice
            .is_some_and(|card| !game.your_hand.contains(&card))
        {
            ui.uno_color_choice = None;
        }
        if ui
            .interaction_menu_open
            .is_some_and(|open| !game.players.iter().any(|player| player.id == open))
        {
            ui.interaction_menu_open = None;
        }
    } else {
        ui.selected.clear();
        ui.card_animations.clear();
        ui.observed_hand.clear();
        ui.greedy_hint.reset();
        ui.interaction_menu_open = None;
        ui.texas_observed_match = None;
        ui.texas_observed_hand_number = 0;
        ui.texas_observed_community_len = 0;
        ui.selected_shengji.clear();
        ui.shengji_card_animations.clear();
        ui.observed_shengji_hand.clear();
        ui.shengji_observed_match = None;
        ui.shengji_observed_hand_number = 0;
        ui.shengji_buried_open = false;
        ui.uno_swap_targets.clear();
        ui.uno_color_choice = None;
        ui.selected_uno.clear();
        ui.uno_card_animations.clear();
    }

    let root = commands
        .spawn((
            UiRoot,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .id();

    add_header(
        &mut commands,
        root,
        client.as_deref(),
        &form,
        &visuals.ui,
        &visuals.avatars,
    );
    if let Some(client) = client.as_deref() {
        if let Some(lobby) = client.0.model().lobby() {
            render_lobby(
                &mut commands,
                root,
                client,
                lobby,
                &ui,
                &visuals.ui,
                &visuals.avatars,
            );
        } else if let Some(game) = client.0.model().qigui523_game() {
            let mut table_visuals = TableVisualContext {
                assets: &visuals.ui,
                avatars: &visuals.avatars,
                appearance: &visuals.table,
                brightness: form.table_brightness,
                vignette: form.table_vignette,
                table_materials: &mut table_materials,
                game_summary: &visuals.game_summary,
                play_effect: &visuals.play_effect,
                score_capture: &visuals.score_capture,
                start_game_transition: &visuals.start_game_transition,
                turn_border_materials: &mut turn_border_materials,
            };
            render_table(
                &mut commands,
                root,
                client,
                game,
                &ui,
                &chat,
                &developer_hand,
                &mut table_visuals,
            );
        } else if let Some(game) = client.0.model().texas_holdem_game() {
            render_texas_holdem_table(
                &mut commands,
                root,
                client,
                game,
                &mut ui,
                &chat,
                TexasTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    start_game_transition: &visuals.start_game_transition,
                    chip_state: &visuals.texas_chips,
                    game_summary: &visuals.game_summary,
                },
            );
        } else if let Some(game) = client.0.model().shengji_game() {
            render_shengji_table(
                &mut commands,
                root,
                client,
                game,
                &mut ui,
                &chat,
                ShengjiTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    start_game_transition: &visuals.start_game_transition,
                    score_capture: &visuals.shengji_score_capture,
                    settlement: &visuals.shengji_settlement,
                    presentation: &visuals.shengji_presentation,
                },
            );
        } else if let Some(game) = client.0.model().uno_game() {
            render_uno_table(
                &mut commands,
                root,
                client,
                game,
                &ui,
                &chat,
                UnoTableVisuals {
                    assets: &visuals.ui,
                    avatars: &visuals.avatars,
                    appearance: &visuals.table,
                    brightness: form.table_brightness,
                    vignette: form.table_vignette,
                    table_materials: &mut table_materials,
                    turn_border_materials: &mut turn_border_materials,
                    game_summary: &visuals.game_summary,
                },
            );
        } else {
            render_connection(
                &mut commands,
                root,
                &form,
                Some(&client.0),
                &visuals.ui,
                &visuals.avatars,
            );
        }
    } else {
        render_connection(
            &mut commands,
            root,
            &form,
            None,
            &visuals.ui,
            &visuals.avatars,
        );
    }
    if ui.settings_open {
        render_settings_modal(&mut commands, root, &form, &visuals.updater, &visuals.ui);
    }
    if ui.profile_open {
        let (name, avatar, reference_points, completed_games, game_profiles) =
            if let Some(player_profile) = ui.player_profile.as_ref() {
                (
                    player_profile.name.as_str(),
                    player_profile.avatar.as_ref(),
                    player_profile.reference_points,
                    player_profile.completed_games,
                    &player_profile.game_profiles,
                )
            } else {
                (
                    form.player_name.as_str(),
                    visuals.avatars.local.as_ref(),
                    profile.reference_points(),
                    profile.completed_games(),
                    profile.game_profiles(),
                )
            };
        render_profile_modal(
            &mut commands,
            root,
            name,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            ui.profile_game_tab,
            &visuals.ui,
        );
    }
    if ui.host_game_picker_open && client.is_none() {
        render_host_game_picker(&mut commands, root, &visuals.ui);
    }
    if visuals.updater.dialog_open {
        render_update_dialog(&mut commands, root, &visuals.updater, &visuals.ui);
    }
    if visuals.play_error.active
        && let Some(message) = visuals.play_error.message.as_deref()
    {
        add_play_error_popup(
            &mut commands,
            root,
            message,
            &visuals.play_error,
            &visuals.ui,
        );
    }
}

fn add_header(
    commands: &mut Commands,
    root: Entity,
    client: Option<&ClientResource>,
    form: &ConnectionForm,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let header = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            height: px(64),
            padding: UiRect::axes(px(24), px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        Some(HEADER_BG),
    );
    let left = spawn_node(
        commands,
        header,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    add_text(commands, left, "LeoCard", 25.0, ACCENT, assets);
    if let Some(host_port) = client.and_then(|client| client.0.model().host_port()) {
        let divider = spawn_node(
            commands,
            left,
            Node {
                width: px(1),
                height: px(24),
                ..default()
            },
            Some(BORDER),
        );
        commands.entity(divider).insert(FocusPolicy::Pass);
        add_text(
            commands,
            left,
            format!("端口 {host_port}"),
            15.0,
            MUTED,
            assets,
        );
    }
    let right = spawn_node(
        commands,
        header,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(14),
            ..default()
        },
        None,
    );
    add_header_button(
        commands,
        right,
        "游戏设置",
        UiAction::ToggleSettings,
        assets,
    );
    if client.is_some_and(|client| client.0.model().game_snapshot().is_some()) {
        add_header_exit_button(commands, right, "退出游戏", assets);
    }
    add_profile_avatar_button(commands, right, form, assets, avatars);
}

fn add_profile_avatar_button(
    commands: &mut Commands,
    parent: Entity,
    form: &ConnectionForm,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let mut entity = commands.spawn((
        Button,
        UiAction::ToggleProfile,
        Node {
            width: px(42),
            height: px(42),
            min_width: px(42),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(percent(50)),
            overflow: Overflow::clip(),
            ..default()
        },
        UiTransform::IDENTITY,
    ));
    if let Some(image) = avatars.local.as_ref() {
        entity.insert(ImageNode::new(image.clone()));
    } else {
        entity.insert(BackgroundColor(avatar_color(&form.player_name)));
    }
    let button = entity.id();
    commands.entity(parent).add_child(button);
    if avatars.local.is_none() {
        add_text(
            commands,
            button,
            form.player_name.chars().next().unwrap_or('玩').to_string(),
            17.5,
            Color::WHITE,
            assets,
        );
    }
}

pub(in crate::app) fn render_profile_modal(
    commands: &mut Commands,
    root: Entity,
    player_name: &str,
    avatar: Option<&Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    game_profiles: &PlayerGameProfiles,
    selected_game: ProfileGameTab,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
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
        Some(Color::srgba(0.005, 0.015, 0.012, 0.76)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2000), FocusPolicy::Block));

    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(820),
            max_width: percent(90),
            min_height: px(520),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "个人资料", assets);

    let identity = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(138),
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(12),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.86)),
    );
    commands.entity(identity).insert(BorderColor::all(BORDER));

    let avatar_area = spawn_node(
        commands,
        identity,
        Node {
            width: px(96),
            min_width: px(96),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(8),
            ..default()
        },
        None,
    );
    add_avatar(commands, avatar_area, player_name, avatar, 82.0, assets);

    let identity_text = spawn_node(
        commands,
        identity,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            row_gap: px(7),
            ..default()
        },
        None,
    );
    add_text(commands, identity_text, player_name, 25.0, TEXT, assets);
    add_profile_interaction_totals(
        commands,
        identity_text,
        game_profiles.interactions.as_ref(),
        assets,
    );

    let stats = spawn_node(
        commands,
        identity,
        Node {
            width: px(300),
            min_width: px(300),
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_profile_stat(
        commands,
        stats,
        "分数",
        reference_points.to_string(),
        assets,
    );
    add_profile_stat(
        commands,
        stats,
        "完成对局",
        completed_games.to_string(),
        assets,
    );
    add_profile_stat(
        commands,
        stats,
        "等级",
        reference_level(reference_points),
        assets,
    );

    add_text(commands, modal, "游戏档案", 16.0, TEXT, assets);
    let tabs = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            height: px(42),
            flex_direction: FlexDirection::Row,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    for (game, label) in ProfileGameTab::ALL {
        add_profile_game_tab(commands, tabs, game, label, game == selected_game, assets);
    }

    let content = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(166),
            flex_grow: 1.0,
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            column_gap: px(12),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.54)),
    );
    commands
        .entity(content)
        .insert((ProfileGameContent, BorderColor::all(BORDER)));
    let rows = match selected_game {
        ProfileGameTab::QiGui523 => qigui523_profile_rows(game_profiles.qigui523.as_ref()),
        ProfileGameTab::TexasHoldem => {
            texas_holdem_profile_rows(game_profiles.texas_holdem.as_ref())
        }
        ProfileGameTab::Shengji => shengji_profile_rows(game_profiles.shengji.as_ref()),
        ProfileGameTab::Uno => uno_profile_rows(game_profiles.uno.as_ref()),
    };
    let rows_per_column = rows.len().div_ceil(4).max(1);
    for column_index in 0..4 {
        let column = spawn_node(
            commands,
            content,
            Node {
                min_width: px(0),
                flex_basis: px(0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                ..default()
            },
            None,
        );
        commands.entity(column).insert(ProfileGameColumn);
        for (label, value) in rows
            .iter()
            .skip(column_index * rows_per_column)
            .take(rows_per_column)
        {
            add_profile_game_row(commands, column, label, value, assets);
        }
    }

    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "关闭",
        UiAction::ToggleProfile,
        ButtonKind::Secondary,
        assets,
    );
}

fn add_profile_game_row(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(22),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(commands, row, label, 12.5, MUTED, assets);
    add_text(commands, row, value, 13.0, TEXT, assets);
}

fn add_profile_game_tab(
    commands: &mut Commands,
    parent: Entity,
    game: ProfileGameTab,
    label: &str,
    selected: bool,
    assets: &UiAssets,
) {
    let normal = if selected {
        Color::srgb(0.36, 0.48, 0.32)
    } else {
        Color::srgb(0.22, 0.32, 0.29)
    };
    let button = commands
        .spawn((
            Button,
            UiAction::SelectProfileGameTab(game),
            ButtonTint {
                normal,
                hovered: if selected {
                    Color::srgb(0.43, 0.55, 0.36)
                } else {
                    Color::srgb(0.30, 0.42, 0.36)
                },
                pressed: Color::srgb(0.18, 0.28, 0.24),
            },
            Node {
                min_width: px(0),
                height: percent(100),
                flex_basis: px(0),
                flex_grow: 1.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::bottom(px(if selected { 3 } else { 1 })),
                border_radius: BorderRadius::top(px(7)),
                ..default()
            },
            ImageNode::new(assets.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(normal),
            BorderColor::all(if selected { ACCENT } else { BORDER }),
            ProfileGameTabButton,
        ))
        .id();
    if selected {
        commands.entity(button).insert(SelectedProfileGameTab);
    }
    commands.entity(parent).add_child(button);
    add_text(
        commands,
        button,
        label,
        14.0,
        if selected { Color::WHITE } else { MUTED },
        assets,
    );
}

fn add_profile_interaction_totals(
    commands: &mut Commands,
    parent: Entity,
    stats: Option<&PlayerInteractionStats>,
    assets: &UiAssets,
) {
    let stats = stats.cloned().unwrap_or_default();
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(28),
            align_items: AlignItems::Center,
            column_gap: px(18),
            ..default()
        },
        None,
    );
    for (kind, count) in [
        (PlayerInteractionKind::Flower, stats.flowers_received),
        (PlayerInteractionKind::Egg, stats.eggs_received),
    ] {
        let item = spawn_node(
            commands,
            row,
            Node {
                align_items: AlignItems::Center,
                column_gap: px(5),
                ..default()
            },
            None,
        );
        let icon = commands
            .spawn((
                Node {
                    width: px(24),
                    height: px(24),
                    ..default()
                },
                ImageNode::new(
                    assets
                        .interaction_images
                        .get(&(kind, false))
                        .cloned()
                        .unwrap_or_default(),
                ),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(item).add_child(icon);
        add_text(commands, item, count.to_string(), 16.0, ACCENT, assets);
    }
}

fn add_profile_stat(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: impl Into<String>,
    assets: &UiAssets,
) {
    let card = spawn_node(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(76),
            flex_basis: px(0),
            flex_grow: 1.0,
            padding: UiRect::axes(px(4), px(10)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(5),
            border: UiRect::ZERO,
            ..default()
        },
        None,
    );
    commands.entity(card).insert(ProfileStat);
    add_text(commands, card, label, 12.0, MUTED, assets);
    add_text(commands, card, value, 21.0, ACCENT, assets);
}

pub(super) fn render_settings_modal(
    commands: &mut Commands,
    root: Entity,
    form: &ConnectionForm,
    updater: &UpdateManager,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
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
        Some(Color::srgba(0.005, 0.015, 0.012, 0.76)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2000), FocusPolicy::Block));
    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(620),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "游戏设置", assets);
    add_text(commands, modal, "自定义桌布背景", 16.0, TEXT, assets);
    let path_box = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(54),
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(10),
            ..default()
        },
        Some(HEADER_BG),
    );
    commands.entity(path_box).insert(BorderColor::all(BORDER));
    let path_text = spawn_node(
        commands,
        path_box,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            ..default()
        },
        None,
    );
    add_text(
        commands,
        path_text,
        form.table_felt_path.as_ref().map_or_else(
            || "当前使用内置深绿色桌布".to_owned(),
            |path| format!("图片路径：{}", path.display()),
        ),
        13.0,
        if form.table_felt_path.is_some() {
            TEXT
        } else {
            MUTED
        },
        assets,
    );
    add_header_button(
        commands,
        path_box,
        "选择图片",
        UiAction::ChooseTableFelt,
        assets,
    );
    for setting in [
        TableAppearanceSetting::Brightness,
        TableAppearanceSetting::Vignette,
        TableAppearanceSetting::Volume,
    ] {
        add_table_appearance_slider(commands, modal, setting, form, assets);
    }
    let update_row = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            min_height: px(68),
            padding: UiRect::axes(px(12), px(10)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(12),
            row_gap: px(8),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.78)),
    );
    commands.entity(update_row).insert(BorderColor::all(BORDER));
    let version_text = spawn_node(
        commands,
        update_row,
        Node {
            min_width: px(180),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
        None,
    );
    add_text(commands, version_text, "软件更新", 16.0, TEXT, assets);
    add_text(
        commands,
        version_text,
        format!("当前版本 v{}", env!("CARGO_PKG_VERSION")),
        12.0,
        MUTED,
        assets,
    );
    let update_actions = spawn_node(
        commands,
        update_row,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(10),
            ..default()
        },
        None,
    );
    add_github_repository_button(commands, update_actions, assets);
    add_green_update_button(
        commands,
        update_actions,
        settings_update_label(&updater.state),
        match updater.state {
            UpdateState::Ready { .. } => UiAction::RestartToUpdate,
            _ => UiAction::StartUpdate,
        },
        assets,
    );
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexEnd,
            column_gap: px(10),
            row_gap: px(8),
            ..default()
        },
        None,
    );
    if form.table_felt_path.is_some() {
        add_action_button(
            commands,
            actions,
            "恢复默认",
            UiAction::UseDefaultTableFelt,
            ButtonKind::Secondary,
            assets,
        );
    }
    add_action_button(
        commands,
        actions,
        "关闭",
        UiAction::ToggleSettings,
        ButtonKind::Secondary,
        assets,
    );
}

fn add_table_appearance_slider(
    commands: &mut Commands,
    parent: Entity,
    setting: TableAppearanceSetting,
    form: &ConnectionForm,
    assets: &UiAssets,
) {
    let fraction = table_appearance_fraction(setting, form);
    let group = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    let label = add_text(
        commands,
        group,
        table_appearance_label(setting, form),
        14.0,
        TEXT,
        assets,
    );
    commands.entity(label).insert(TableAppearanceLabel(setting));
    let slider = commands
        .spawn((
            Button,
            TableAppearanceSlider(setting),
            RelativeCursorPosition::default(),
            Node {
                width: percent(100),
                height: px(34),
                position_type: PositionType::Relative,
                ..default()
            },
        ))
        .id();
    commands.entity(group).add_child(slider);
    let track = spawn_node(
        commands,
        slider,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(13),
            height: px(8),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        Some(HEADER_BG),
    );
    let fill = spawn_node(
        commands,
        track,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(fraction * 100.0),
            height: percent(100),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        Some(Color::srgb(0.12, 0.48, 0.70)),
    );
    commands.entity(fill).insert(TableAppearanceIndicator {
        setting,
        part: TableAppearanceIndicatorPart::Fill,
    });
    let knob = spawn_node(
        commands,
        slider,
        Node {
            position_type: PositionType::Absolute,
            left: percent(fraction * 100.0),
            top: px(8),
            width: px(18),
            height: px(18),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            ..default()
        },
        Some(TEXT),
    );
    commands.entity(knob).insert((
        TableAppearanceIndicator {
            setting,
            part: TableAppearanceIndicatorPart::Knob,
        },
        BorderColor::all(Color::srgb(0.12, 0.48, 0.70)),
        UiTransform::from_translation(Val2::px(-9.0, 0.0)),
        FocusPolicy::Pass,
    ));
}

pub(super) const HOST_GAME_CHOICES: [(&str, &str, GameKind); 4] = [
    ("七鬼五二三", "放空大脑, 有牌就出", GameKind::QiGui523),
    ("德州扑克", "窝要验牌!", GameKind::TexasHoldem),
    ("升级", "神对手 or 猪队友", GameKind::Shengji),
    ("UNO", "最后一张，记得喊 UNO!", GameKind::Uno),
];

fn render_host_game_picker(commands: &mut Commands, root: Entity, assets: &UiAssets) {
    let overlay = spawn_node(
        commands,
        root,
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
        Some(Color::srgba(0.005, 0.015, 0.012, 0.78)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2100), FocusPolicy::Block));

    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(1050),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "选择游戏", assets);
    add_text(
        commands,
        modal,
        "选择本房间要进行的棋牌游戏。进入等待大厅后可继续配置该游戏的规则。",
        14.0,
        MUTED,
        assets,
    );

    let choices = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            column_gap: px(16),
            ..default()
        },
        None,
    );
    for (title, description, game) in HOST_GAME_CHOICES {
        add_host_game_choice(commands, choices, title, description, Some(game), assets);
    }

    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "取消",
        UiAction::CloseHostGamePicker,
        ButtonKind::Secondary,
        assets,
    );
}

fn add_host_game_choice(
    commands: &mut Commands,
    parent: Entity,
    title: &str,
    description: &str,
    game: Option<GameKind>,
    assets: &UiAssets,
) {
    let card = add_panel(
        commands,
        parent,
        Node {
            min_width: px(0),
            min_height: px(190),
            flex_basis: px(0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            row_gap: px(12),
            ..default()
        },
        if game.is_some() {
            PANEL_ALT
        } else {
            HEADER_BG.with_alpha(0.72)
        },
        PanelSkin::Section,
        assets,
    );
    let copy = spawn_node(
        commands,
        card,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        None,
    );
    add_text(
        commands,
        copy,
        title,
        24.0,
        if game.is_some() { ACCENT } else { MUTED },
        assets,
    );
    add_text(commands, copy, description, 14.0, MUTED, assets);
    if let Some(game) = game {
        add_action_button(
            commands,
            card,
            "创建房间",
            UiAction::CreateRoom(game),
            ButtonKind::Primary,
            assets,
        );
    } else {
        add_disabled_action_button(commands, card, "尚未接入", assets);
    }
}

fn render_connection(
    commands: &mut Commands,
    root: Entity,
    form: &ConnectionForm,
    network: Option<&TcpGameClient>,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1050),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(28)),
            flex_direction: FlexDirection::Column,
            row_gap: px(18),
            justify_content: JustifyContent::Center,
            ..default()
        },
        None,
    );

    let profile_row = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(28),
            ..default()
        },
        None,
    );
    let name_field = spawn_node(
        commands,
        profile_row,
        Node {
            width: px(260),
            max_width: px(260),
            min_width: px(220),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        None,
    );
    add_input(
        commands,
        name_field,
        "玩家名称",
        &form.player_name,
        InputField::PlayerName,
        form.active == InputField::PlayerName,
        assets,
    );
    let avatar_row = spawn_node(
        commands,
        profile_row,
        Node {
            width: px(365),
            min_width: px(300),
            flex_shrink: 0.0,
            min_height: px(58),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let avatar_button = commands
        .spawn((
            Button,
            UiAction::ChooseAvatar,
            Node {
                width: px(54),
                height: px(54),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(percent(50)),
                ..default()
            },
            BackgroundColor(PANEL_ALT),
            BorderColor::all(ACCENT.with_alpha(0.7)),
        ))
        .id();
    commands.entity(avatar_row).add_child(avatar_button);
    add_avatar(
        commands,
        avatar_button,
        &form.player_name,
        avatars.local.as_ref(),
        48.0,
        assets,
    );
    let avatar_help = spawn_node(
        commands,
        avatar_row,
        Node {
            min_width: px(0),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            ..default()
        },
        None,
    );
    add_text(commands, avatar_help, "个人头像", 14.0, TEXT, assets);
    add_text(
        commands,
        avatar_help,
        "点击头像更换图片",
        12.0,
        MUTED,
        assets,
    );
    if form.avatar_png.is_some() {
        add_action_button(
            commands,
            avatar_row,
            "清除头像",
            UiAction::ClearAvatar,
            ButtonKind::Secondary,
            assets,
        );
    }

    let choices = spawn_node(
        commands,
        content,
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(18),
            row_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let host = add_panel(
        commands,
        choices,
        Node {
            min_width: px(360),
            flex_basis: px(470),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, host, "开设房间", assets);
    add_text(
        commands,
        host,
        "本机将监听所有局域网网卡",
        13.0,
        MUTED,
        assets,
    );
    add_input(
        commands,
        host,
        "监听端口",
        &form.host_port,
        InputField::HostPort,
        form.active == InputField::HostPort,
        assets,
    );
    add_action_button(
        commands,
        host,
        "选择游戏并创建",
        UiAction::OpenHostGamePicker,
        ButtonKind::Primary,
        assets,
    );

    let join = add_panel(
        commands,
        choices,
        Node {
            min_width: px(360),
            flex_basis: px(470),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(commands, join, "加入房间", assets);
    add_text(
        commands,
        join,
        "输入房主的局域网地址，例如 192.168.1.20:52300。",
        13.0,
        MUTED,
        assets,
    );
    add_input(
        commands,
        join,
        "连接地址",
        &form.join_address,
        InputField::JoinAddress,
        form.active == InputField::JoinAddress,
        assets,
    );
    add_action_button(
        commands,
        join,
        "连接并加入",
        UiAction::JoinRoom,
        ButtonKind::Warning,
        assets,
    );

    if let Some((status, color)) = network.and_then(|network| match network.state() {
        NetworkState::Connecting(message) | NetworkState::Reconnecting(message) => {
            Some((message.as_str(), ACCENT))
        }
        NetworkState::Failed(message) => Some((message.as_str(), DANGER)),
        NetworkState::Connected(_) => None,
    }) {
        add_text(commands, content, status, 15.0, color, assets);
    }
}

fn add_input(
    commands: &mut Commands,
    parent: Entity,
    label: &str,
    value: &str,
    field: InputField,
    active: bool,
    assets: &UiAssets,
) {
    add_text(commands, parent, label, 13.0, MUTED, assets);
    let input = commands
        .spawn((
            Button,
            UiAction::FocusInput(field),
            Node {
                width: percent(100),
                min_height: px(45),
                padding: UiRect::axes(px(13), px(9)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(if active { 2 } else { 1 })),
                border_radius: BorderRadius::all(px(5)),
                ..default()
            },
            BackgroundColor(HEADER_BG),
            BorderColor::all(if active { ACCENT } else { BORDER }),
        ))
        .id();
    commands.entity(parent).add_child(input);
    add_text(
        commands,
        input,
        format!("{value}{}", if active { "│" } else { "" }),
        16.0,
        if value.is_empty() { MUTED } else { TEXT },
        assets,
    );
}

fn render_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    ui: &UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    if lobby.game == GameKind::Uno {
        render_uno_lobby(commands, root, client, lobby, ui, assets, avatars);
        return;
    }
    if lobby.game == GameKind::TexasHoldem {
        render_texas_holdem_lobby(commands, root, client, lobby, assets, avatars);
        return;
    }
    if lobby.game == GameKind::Shengji {
        render_shengji_lobby(commands, root, client, lobby, assets, avatars);
        return;
    }
    let game_rules = *lobby
        .qigui523_rules()
        .expect("七鬼五二三大厅应携带对应规则");
    let connected_count = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(18),
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );

    let rules = add_panel(
        commands,
        content,
        Node {
            min_width: px(300),
            flex_basis: px(330),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    add_section_title(commands, rules, "游戏配置", assets);
    add_text(
        commands,
        rules,
        format!("当前人数 {connected_count}/{TABLE_SEAT_COUNT}。"),
        13.0,
        MUTED,
        assets,
    );

    let deck_previous = (game_rules.deck_count > RuleSet::MIN_DECK_COUNT)
        .then(|| RuleSet {
            deck_count: game_rules.deck_count - 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    let deck_next = (game_rules.deck_count < RuleSet::MAX_DECK_COUNT)
        .then(|| RuleSet {
            deck_count: game_rules.deck_count + 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "牌副数",
            value: format!("{} 副", game_rules.deck_count),
            help: "使用几副完整扑克牌，可设置 1–8 副。多副牌会出现同花色同点数的重复牌。",
            editable: can_configure,
            previous: deck_previous.filter(|_| can_configure),
            next: deck_next.filter(|_| can_configure),
        },
        assets,
    );

    #[cfg(feature = "developer")]
    {
        let toggled_developer_deck = RuleSet {
            developer_deck: !game_rules.developer_deck,
            ..game_rules
        };
        add_rule_config_row(
            commands,
            rules,
            RuleConfigRow {
                label: "开发者牌堆",
                value: if game_rules.developer_deck {
                    "开启".to_owned()
                } else {
                    "关闭".to_owned()
                },
                help: "开启后摸牌堆只生成所有玩家的初始手牌；关闭则使用完整牌堆。",
                editable: can_configure,
                previous: can_configure.then_some(toggled_developer_deck),
                next: can_configure.then_some(toggled_developer_deck),
            },
            assets,
        );
    }

    let hand_previous = (game_rules.hand_size > RuleSet::MIN_HAND_SIZE)
        .then(|| RuleSet {
            hand_size: game_rules.hand_size - 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    let hand_next = (game_rules.hand_size < RuleSet::MAX_HAND_SIZE)
        .then(|| RuleSet {
            hand_size: game_rules.hand_size + 1,
            ..game_rules
        })
        .filter(|rules| rules.validate().is_ok());
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "手牌张数",
            value: format!("{} 张", game_rules.hand_size),
            help: "每轮结束后，仍有摸牌时所有玩家会把手牌补到该数量，可设置 5–15 张。",
            editable: can_configure,
            previous: hand_previous.filter(|_| can_configure),
            next: hand_next.filter(|_| can_configure),
        },
        assets,
    );

    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "出牌计时",
            value: time_control_label(game_rules.time_control).to_owned(),
            help: "不限时不会倒计时；X+Y 表示每次轮到玩家时重置 X 秒，之后扣除该玩家本局共享的 Y 秒，全部耗尽后自动出牌。",
            editable: can_configure,
            previous: can_configure
                .then(|| previous_time_control(game_rules.time_control))
                .flatten()
                .map(|time_control| RuleSet {
                    time_control,
                    ..game_rules
                }),
            next: can_configure
                .then(|| next_time_control(game_rules.time_control))
                .flatten()
                .map(|time_control| RuleSet {
                    time_control,
                    ..game_rules
                }),
        },
        assets,
    );

    let suit_previous = RuleSet {
        suit_comparison: previous_suit_comparison(game_rules.suit_comparison),
        ..game_rules
    };
    let suit_next = RuleSet {
        suit_comparison: next_suit_comparison(game_rules.suit_comparison),
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "花色比较",
            value: suit_comparison_label(game_rules.suit_comparison).to_owned(),
            help: "点数组成相同时：极大法只比最大牌；逐项法从最大牌依次比较；记点法按黑桃/红桃/梅花/方块 4/3/2/1 点求和。",
            editable: can_configure,
            previous: can_configure.then_some(suit_previous),
            next: can_configure.then_some(suit_next),
        },
        assets,
    );

    let toggled_policy = RuleSet {
        same_card_policy: match game_rules.same_card_policy {
            SameCardPolicy::MustBeHigher => SameCardPolicy::CanFollow,
            SameCardPolicy::CanFollow => SameCardPolicy::MustBeHigher,
        },
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "同强度跟牌",
            value: match game_rules.same_card_policy {
                SameCardPolicy::MustBeHigher => "不允许".to_owned(),
                SameCardPolicy::CanFollow => "允许".to_owned(),
            },
            help: "整手牌比较结果完全相同时，决定后出的玩家是否仍可跟牌。关闭时必须严格更大。",
            editable: can_configure,
            previous: can_configure.then_some(toggled_policy),
            next: can_configure.then_some(toggled_policy),
        },
        assets,
    );

    let toggled_advanced_play_types = RuleSet {
        advanced_play_types: !game_rules.advanced_play_types,
        ..game_rules
    };
    add_rule_config_row(
        commands,
        rules,
        RuleConfigRow {
            label: "牌型进阶",
            value: if game_rules.advanced_play_types {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后允许三带一和三带一对；纯三张压任意三张顺子；两连对压任意四张顺子；两连飞机压任意三连对。三带牌不能压顺子，其他关系也不能反向压制。",
            editable: can_configure,
            previous: can_configure.then_some(toggled_advanced_play_types),
            next: can_configure.then_some(toggled_advanced_play_types),
        },
        assets,
    );
    let players = add_panel(
        commands,
        content,
        Node {
            min_width: px(380),
            flex_basis: px(560),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players,
        format!("玩家席位  {}/{}", connected_count, game_rules.player_count),
        assets,
    );
    render_seat_selector(commands, players, client, lobby, assets, avatars);

    let actions = spawn_node(
        commands,
        players,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert((GlobalZIndex(800), FocusPolicy::Pass));
    let ready = client
        .0
        .model()
        .you()
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = client.0.model().you() == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if !is_host {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    } else {
        let can_start = connected_count >= 2
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待玩家中", assets);
        }
    }
}

fn render_uno_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    ui: &UiState,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby.uno_rules().expect("UNO 大厅应携带对应规则");
    let connected_count = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(18),
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let rules_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(300),
            flex_basis: px(330),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    let title_row = spawn_node(
        commands,
        rules_panel,
        Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(12),
            ..default()
        },
        None,
    );
    add_section_title(
        commands,
        title_row,
        if rules_value.is_no_mercy() {
            "No Mercy 配置"
        } else if rules_value.is_flip() {
            "UNO FLIP 配置"
        } else {
            "UNO 配置"
        },
        assets,
    );
    let switched = UnoRuleSet {
        mode: match rules_value.mode {
            leocard_uno::Mode::Classic => leocard_uno::Mode::NoMercy,
            leocard_uno::Mode::NoMercy => leocard_uno::Mode::Flip,
            leocard_uno::Mode::Flip => leocard_uno::Mode::Classic,
        },
        ..rules_value
    };
    if can_configure {
        add_action_button(
            commands,
            title_row,
            match rules_value.mode {
                leocard_uno::Mode::Classic => "切换至 No Mercy",
                leocard_uno::Mode::NoMercy => "切换至 UNO FLIP",
                leocard_uno::Mode::Flip => "切换至 UNO",
            },
            UiAction::UpdateUnoRules(switched),
            ButtonKind::Warning,
            assets,
        );
    } else {
        add_disabled_action_button(
            commands,
            title_row,
            match rules_value.mode {
                leocard_uno::Mode::Classic => "UNO 模式",
                leocard_uno::Mode::NoMercy => "No Mercy 模式",
                leocard_uno::Mode::Flip => "UNO FLIP 模式",
            },
            assets,
        );
    }
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected_count}/{}", UnoRuleSet::MAX_PLAYERS),
        13.0,
        MUTED,
        assets,
    );
    if rules_value.is_classic() {
        let stack_toggled = UnoRuleSet {
            action_stacking: !rules_value.action_stacking,
            ..rules_value
        };
        add_uno_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "功能牌堆叠",
                value: if rules_value.action_stacking {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "允许禁手、+2、万能 +4 及兼容的扩展功能牌继续累计；+4 可压在 +2 上，+2 不能反压 +4。",
                editable: can_configure,
                previous: can_configure.then_some(stack_toggled),
                next: can_configure.then_some(stack_toggled),
            },
            assets,
        );
    } else if rules_value.is_no_mercy() {
        for (label, enabled, help, toggled) in [
            (
                "摸到能出",
                rules_value.no_mercy.draw_until_playable,
                "无牌可出时持续摸牌，直到摸到一张可出的牌，并必须处理该牌。",
                UnoRuleSet {
                    no_mercy: leocard_uno::NoMercyRuleSet {
                        draw_until_playable: !rules_value.no_mercy.draw_until_playable,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "慈悲淘汰",
                rules_value.no_mercy.mercy_elimination,
                "手牌达到 25 张时立即淘汰；只剩一名未淘汰玩家时结束。",
                UnoRuleSet {
                    no_mercy: leocard_uno::NoMercyRuleSet {
                        mercy_elimination: !rules_value.no_mercy.mercy_elimination,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "0 传递手牌",
                rules_value.no_mercy.zero_pass,
                "打出 0 时，所有未淘汰玩家按当前方向传递整手牌。",
                UnoRuleSet {
                    no_mercy: leocard_uno::NoMercyRuleSet {
                        zero_pass: !rules_value.no_mercy.zero_pass,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "7 交换手牌",
                rules_value.no_mercy.seven_swap,
                "打出 7 后选择一名未淘汰玩家并与其交换整手牌。",
                UnoRuleSet {
                    no_mercy: leocard_uno::NoMercyRuleSet {
                        seven_swap: !rules_value.no_mercy.seven_swap,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
            (
                "UNO 宣告与检举",
                rules_value.no_mercy.uno_callout,
                "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                UnoRuleSet {
                    no_mercy: leocard_uno::NoMercyRuleSet {
                        uno_callout: !rules_value.no_mercy.uno_callout,
                        ..rules_value.no_mercy
                    },
                    ..rules_value
                },
            ),
        ] {
            add_uno_rule_config_row(
                commands,
                rules_panel,
                UnoRuleConfigRow {
                    label,
                    value: if enabled { "开启" } else { "关闭" }.to_owned(),
                    help,
                    editable: can_configure,
                    previous: can_configure.then_some(toggled),
                    next: can_configure.then_some(toggled),
                },
                assets,
            );
        }
    }
    if rules_value.is_classic() {
        let skip_draw_toggled = UnoRuleSet {
            skip_draw_penalty: !rules_value.skip_draw_penalty,
            ..rules_value
        };
        add_uno_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "禁手摸牌",
                value: if rules_value.skip_draw_penalty {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "玩家每实际跳过一轮时，额外摸一张牌。",
                editable: can_configure,
                previous: can_configure.then_some(skip_draw_toggled),
                next: can_configure.then_some(skip_draw_toggled),
            },
            assets,
        );
        let jump_in_toggled = UnoRuleSet {
            jump_in: !rules_value.jump_in,
            ..rules_value
        };
        add_uno_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "抢出",
                value: if rules_value.jump_in {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "彩色牌落桌后，非下家若持有颜色和牌面完全相同的另一张牌，可在下家执行动作前抢出。关闭功能牌堆叠时只可抢数字牌。相同双牌可一次打出。",
                editable: can_configure,
                previous: can_configure.then_some(jump_in_toggled),
                next: can_configure.then_some(jump_in_toggled),
            },
            assets,
        );
        let callout_toggled = UnoRuleSet {
            uno_callout: !rules_value.uno_callout,
            ..rules_value
        };
        add_uno_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "UNO 宣告与检举",
                value: if rules_value.uno_callout {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "手里恰好两张且轮到自己时可先喊 UNO，随后本回合必须出到一张；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                editable: can_configure,
                previous: can_configure.then_some(callout_toggled),
                next: can_configure.then_some(callout_toggled),
            },
            assets,
        );
    } else if rules_value.is_flip() {
        for (label, enabled, help, toggled) in [
            (
                "随机正反配对",
                rules_value.flip.random_pairing,
                "关闭时使用固定的正反面组合；开启后每局重新随机配对全部 112 张牌的两面。",
                UnoRuleSet {
                    flip: leocard_uno::FlipRuleSet {
                        random_pairing: !rules_value.flip.random_pairing,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "功能牌堆叠",
                rules_value.flip.action_stacking,
                "允许亮暗两面的禁手与罚牌继续累计；各罚牌链仍按对应牌型规则结算。",
                UnoRuleSet {
                    flip: leocard_uno::FlipRuleSet {
                        action_stacking: !rules_value.flip.action_stacking,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "禁手摸牌",
                rules_value.flip.skip_draw_penalty,
                "玩家每实际跳过一轮时，额外摸一张牌。",
                UnoRuleSet {
                    flip: leocard_uno::FlipRuleSet {
                        skip_draw_penalty: !rules_value.flip.skip_draw_penalty,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
            (
                "UNO 宣告与检举",
                rules_value.flip.uno_callout,
                "手里恰好两张且轮到自己时可先喊 UNO；未喊直接出到一张者在下次成功出牌前可被检举并罚摸 2 张。",
                UnoRuleSet {
                    flip: leocard_uno::FlipRuleSet {
                        uno_callout: !rules_value.flip.uno_callout,
                        ..rules_value.flip
                    },
                    ..rules_value
                },
            ),
        ] {
            add_uno_rule_config_row(
                commands,
                rules_panel,
                UnoRuleConfigRow {
                    label,
                    value: if enabled { "开启" } else { "关闭" }.to_owned(),
                    help,
                    editable: can_configure,
                    previous: can_configure.then_some(toggled),
                    next: can_configure.then_some(toggled),
                },
                assets,
            );
        }
        let toggled = UnoRuleSet {
            flip: leocard_uno::FlipRuleSet {
                jump_in: !rules_value.flip.jump_in,
                ..rules_value.flip
            },
            ..rules_value
        };
        add_uno_rule_config_row(
            commands,
            rules_panel,
            UnoRuleConfigRow {
                label: "抢出",
                value: if rules_value.flip.jump_in {
                    "开启"
                } else {
                    "关闭"
                }
                .to_owned(),
                help: "只比较当前牌面；关闭功能牌堆叠时只可抢数字牌，相同双牌可一次打出。",
                editable: can_configure,
                previous: can_configure.then_some(toggled),
                next: can_configure.then_some(toggled),
            },
            assets,
        );
    }
    let expansion_button = add_action_button(
        commands,
        rules_panel,
        "扩展包设置",
        UiAction::ToggleUnoExpansionSettings,
        ButtonKind::Secondary,
        assets,
    );
    commands.entity(expansion_button).insert(Node {
        width: percent(100),
        min_width: px(0),
        height: px(56),
        padding: UiRect::axes(px(18), px(8)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    let players = add_panel(
        commands,
        content,
        Node {
            min_width: px(380),
            flex_basis: px(560),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players,
        format!("玩家席位  {connected_count}/{}", UnoRuleSet::MAX_PLAYERS),
        assets,
    );
    render_seat_selector(commands, players, client, lobby, assets, avatars);
    let actions = spawn_node(
        commands,
        players,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert((GlobalZIndex(800), FocusPolicy::Pass));
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected_count >= usize::from(UnoRuleSet::MIN_PLAYERS)
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待玩家中", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
    if ui.uno_expansion_settings_open {
        render_uno_expansion_settings(commands, root, rules_value, can_configure, assets);
    }
}

pub(in crate::app) fn render_uno_expansion_settings(
    commands: &mut Commands,
    root: Entity,
    rules: UnoRuleSet,
    can_configure: bool,
    assets: &UiAssets,
) {
    let overlay = spawn_node(
        commands,
        root,
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
        Some(Color::BLACK.with_alpha(0.62)),
    );
    commands
        .entity(overlay)
        .insert((GlobalZIndex(2100), FocusPolicy::Block));
    let modal = add_panel(
        commands,
        overlay,
        Node {
            width: px(620),
            max_width: percent(92),
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Window,
        assets,
    );
    add_section_title(commands, modal, "扩展包设置", assets);
    add_text(
        commands,
        modal,
        if rules.is_no_mercy() {
            "No Mercy 使用独立的扩展包设置。"
        } else if rules.is_flip() {
            "UNO FLIP 使用独立的扩展包设置。"
        } else {
            "选择要加入本房间牌堆的可选扩展包。"
        },
        13.0,
        MUTED,
        assets,
    );
    if rules.is_classic() {
        add_uno_expansion_row(
            commands,
            modal,
            "Swap Pack",
            "以交换手牌为特色，你的手牌随时可能变成别人的",
            rules.swap_pack,
            UnoRuleSet {
                swap_pack: !rules.swap_pack,
                ..rules
            },
            can_configure,
            assets,
        );
        add_uno_expansion_row(
            commands,
            modal,
            "Reverse Pack",
            "以改变方向为特色，小心罚牌反弹——你可能会被自己罚到！",
            rules.reverse_pack,
            UnoRuleSet {
                reverse_pack: !rules.reverse_pack,
                ..rules
            },
            can_configure,
            assets,
        );
        add_uno_expansion_row(
            commands,
            modal,
            "Stack Pack",
            "以累计罚牌为特色，加入堆叠 +1、+2、万能 +3 与随机堆叠牌",
            rules.stack_pack,
            UnoRuleSet {
                stack_pack: !rules.stack_pack,
                ..rules
            },
            can_configure,
            assets,
        );
    } else {
        add_text(
            commands,
            modal,
            if rules.is_flip() {
                "当前尚未加入 UNO FLIP 扩展包。"
            } else {
                "当前尚未加入 No Mercy 扩展包。"
            },
            15.0,
            TEXT,
            assets,
        );
    }
    let actions = spawn_node(
        commands,
        modal,
        Node {
            width: percent(100),
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    add_action_button(
        commands,
        actions,
        "关闭",
        UiAction::ToggleUnoExpansionSettings,
        ButtonKind::Secondary,
        assets,
    );
}

#[allow(clippy::too_many_arguments)]
fn add_uno_expansion_row(
    commands: &mut Commands,
    parent: Entity,
    name: &str,
    description: &str,
    enabled: bool,
    toggled_rules: UnoRuleSet,
    editable: bool,
    assets: &UiAssets,
) {
    let row = spawn_node(
        commands,
        parent,
        Node {
            width: percent(100),
            min_height: px(86),
            padding: UiRect::all(px(13)),
            align_items: AlignItems::Center,
            column_gap: px(14),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(PANEL_ALT.with_alpha(0.86)),
    );
    commands.entity(row).insert(BorderColor::all(BORDER));
    let name_slot = spawn_node(
        commands,
        row,
        Node {
            width: px(112),
            flex_shrink: 0.0,
            ..default()
        },
        None,
    );
    add_text(commands, name_slot, name, 16.0, TEXT, assets);
    add_uno_expansion_status(commands, row, enabled, toggled_rules, editable, assets);
    let description_slot = spawn_node(
        commands,
        row,
        Node {
            min_width: px(0),
            flex_grow: 1.0,
            ..default()
        },
        None,
    );
    add_text(commands, description_slot, description, 13.0, MUTED, assets);
}

fn add_uno_expansion_status(
    commands: &mut Commands,
    parent: Entity,
    enabled: bool,
    toggled_rules: UnoRuleSet,
    editable: bool,
    assets: &UiAssets,
) {
    let mut status = commands.spawn((
        UnoExpansionStatus,
        Node {
            width: px(38),
            height: px(38),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
    if editable {
        status.insert((
            Button,
            UiAction::UpdateUnoRules(toggled_rules),
            UnoExpansionStatusFrame,
            BorderColor::all(if enabled {
                READY.with_alpha(0.82)
            } else {
                DANGER.with_alpha(0.82)
            }),
            BackgroundColor(HEADER_BG.with_alpha(0.92)),
            Node {
                width: px(38),
                height: px(38),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
        ));
    }
    let status = status.id();
    commands.entity(parent).add_child(status);
    add_text(
        commands,
        status,
        if enabled { "✓" } else { "×" },
        24.0,
        if enabled { READY } else { DANGER },
        assets,
    );
}

fn render_shengji_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby.shengji_rules().expect("双升大厅应携带对应规则");
    let connected_count = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(18),
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let rules_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(300),
            flex_basis: px(330),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    add_section_title(commands, rules_panel, "双升配置", assets);
    add_text(
        commands,
        rules_panel,
        format!("当前人数 {connected_count}/4，需要四人开局。"),
        13.0,
        MUTED,
        assets,
    );
    let previous_deck_count = (rules_value.deck_count > 2).then_some(ShengjiRuleSet {
        deck_count: rules_value.deck_count - 1,
        ..rules_value
    });
    let next_deck_count = (rules_value.deck_count < 4).then_some(ShengjiRuleSet {
        deck_count: rules_value.deck_count + 1,
        ..rules_value
    });
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "牌副数",
            value: format!("{} 副", rules_value.deck_count),
            help: "两副牌：25 张手牌、8 张底牌；三副牌：39 张手牌、6 张底牌，加入三同张和泰坦尼克；四副牌：52 张手牌、8 张底牌，再加入炸弹和宇宙飞船。",
            editable: can_configure,
            previous: previous_deck_count.filter(|_| can_configure),
            next: next_deck_count.filter(|_| can_configure),
        },
        assets,
    );
    let throw_toggled = ShengjiRuleSet {
        allow_throw: !rules_value.allow_throw,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "允许甩牌",
            value: if rules_value.allow_throw {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后首家可以甩出同门的单张、对子和拖拉机组合；甩牌失败会被强制改出最小可失败牌型。",
            editable: can_configure,
            previous: can_configure.then_some(throw_toggled),
            next: can_configure.then_some(throw_toggled),
        },
        assets,
    );
    let bottom_copy_toggled = ShengjiRuleSet {
        bottom_copy: !rules_value.bottom_copy,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "抄底",
            value: if rules_value.bottom_copy {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "庄家埋底后，从庄家下家开始依次询问可反主的玩家；每次抄底都公开反主牌、取得当前底牌并重新埋底，然后继续询问，直到一整轮无人再抄底。抄底永远不改变庄家，但实际通过扳底定庄的对局禁用。",
            editable: can_configure,
            previous: can_configure.then_some(bottom_copy_toggled),
            next: can_configure.then_some(bottom_copy_toggled),
        },
        assets,
    );
    let penalties = [
        ShengjiThrowPenalty::None,
        ShengjiThrowPenalty::FivePerCard,
        ShengjiThrowPenalty::TenPerCard,
    ];
    let penalty_index = penalties
        .iter()
        .position(|penalty| *penalty == rules_value.throw_penalty)
        .unwrap_or(0);
    let previous_penalty = penalties[(penalty_index + penalties.len() - 1) % penalties.len()];
    let next_penalty = penalties[(penalty_index + 1) % penalties.len()];
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "甩牌罚分",
            value: match rules_value.throw_penalty {
                ShengjiThrowPenalty::None => "不罚分",
                ShengjiThrowPenalty::FivePerCard => "每张 5 分",
                ShengjiThrowPenalty::TenPerCard => "每张 10 分",
            }
            .to_owned(),
            help: "庄家方罚分会给闲家加分；闲家方罚分从闲家总分扣除，最低为零。",
            editable: can_configure,
            previous: can_configure.then_some(ShengjiRuleSet {
                throw_penalty: previous_penalty,
                ..rules_value
            }),
            next: can_configure.then_some(ShengjiRuleSet {
                throw_penalty: next_penalty,
                ..rules_value
            }),
        },
        assets,
    );
    let mandatory_toggled = ShengjiRuleSet {
        mandatory_five_ten_king_ace: !rules_value.mandatory_five_ten_king_ace,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "必打 5/10/K/A",
            value: if rules_value.mandatory_five_ten_king_ace {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后升级跨过 5、10、K、A 时必须先停在对应等级。",
            editable: can_configure,
            previous: can_configure.then_some(mandatory_toggled),
            next: can_configure.then_some(mandatory_toggled),
        },
        assets,
    );
    let bid_with_joker_toggled = ShengjiRuleSet {
        bid_with_joker: !rules_value.bid_with_joker,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "带王亮",
            value: if rules_value.bid_with_joker {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后红桃/方块必须带一张大王，梅花/黑桃必须带一张小王；本人已经亮过的王可以复用。无主不能首亮，只能用于反主。",
            editable: can_configure,
            previous: can_configure.then_some(bid_with_joker_toggled),
            next: can_configure.then_some(bid_with_joker_toggled),
        },
        assets,
    );
    let power_outage_dealer_toggled = ShengjiRuleSet {
        power_outage_dealer: !rules_value.power_outage_dealer,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "断电换庄",
            value: if rules_value.power_outage_dealer {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "第一次无人亮主时保持手牌不动，由当前庄家的下家直接接庄，改打新庄家一方的级牌并重新亮主 10 秒；不产生上台积分。",
            editable: can_configure,
            previous: can_configure.then_some(power_outage_dealer_toggled),
            next: can_configure.then_some(power_outage_dealer_toggled),
        },
        assets,
    );
    let bottom_flip_toggled = ShengjiRuleSet {
        bottom_flip: !rules_value.bottom_flip,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "扳底",
            value: if rules_value.bottom_flip {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "最终无人亮主时逐张翻开底牌，公开各玩家持有的同牌并按两队总数、队内数量和座次确定庄家；王定无主，其他牌定其花色。",
            editable: can_configure,
            previous: can_configure.then_some(bottom_flip_toggled),
            next: can_configure.then_some(bottom_flip_toggled),
        },
        assets,
    );
    let five_trump_crossing_toggled = ShengjiRuleSet {
        five_trump_crossing: !rules_value.five_trump_crossing,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "五主过江",
            value: if rules_value.five_trump_crossing {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "有主局埋底后，主牌不多于五张的玩家可将全部主牌补足五张交给对家，再由对家任选五张归还；每人每局仅可进行一次。",
            editable: can_configure,
            previous: can_configure.then_some(five_trump_crossing_toggled),
            next: can_configure.then_some(five_trump_crossing_toggled),
        },
        assets,
    );
    let constant_trump_toggled = ShengjiRuleSet {
        constant_trump: !rules_value.constant_trump,
        ..rules_value
    };
    add_shengji_rule_config_row(
        commands,
        rules_panel,
        ShengjiRuleConfigRow {
            label: "常主 2",
            value: if rules_value.constant_trump {
                "开启"
            } else {
                "关闭"
            }
            .to_owned(),
            help: "开启后双方从 3 开始，2 永远为主牌；牌力位于本局级牌和主牌 A 之间，并区分主 2 与副 2。",
            editable: can_configure,
            previous: can_configure.then_some(constant_trump_toggled),
            next: can_configure.then_some(constant_trump_toggled),
        },
        assets,
    );

    let players = add_panel(
        commands,
        content,
        Node {
            min_width: px(380),
            flex_basis: px(560),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players,
        format!("玩家席位  {connected_count}/4"),
        assets,
    );
    render_seat_selector(commands, players, client, lobby, assets, avatars);

    let actions = spawn_node(
        commands,
        players,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert((GlobalZIndex(800), FocusPolicy::Pass));
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected_count == ShengjiRuleSet::PLAYER_COUNT
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待四名玩家", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
}

fn render_texas_holdem_lobby(
    commands: &mut Commands,
    root: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let rules_value = *lobby
        .texas_holdem_rules()
        .expect("德州扑克大厅应携带对应规则");
    let connected_count = connected_lobby_player_count(lobby);
    let content = spawn_node(
        commands,
        root,
        Node {
            width: percent(100),
            max_width: px(1180),
            flex_grow: 1.0,
            align_self: AlignSelf::Center,
            padding: UiRect::all(px(22)),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(18),
            column_gap: px(18),
            align_items: AlignItems::Stretch,
            ..default()
        },
        None,
    );
    let rules_panel = add_panel(
        commands,
        content,
        Node {
            min_width: px(300),
            flex_basis: px(330),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(14),
            ..default()
        },
        PANEL,
        PanelSkin::Section,
        assets,
    );
    let can_configure = client.0.model().you() == lobby.host;
    add_section_title(commands, rules_panel, "德州扑克配置", assets);
    add_text(
        commands,
        rules_panel,
        format!(
            "当前人数 {}/{}，至少 3 人开局。",
            connected_count, TABLE_SEAT_COUNT
        ),
        13.0,
        MUTED,
        assets,
    );

    let chip_index = TexasHoldemRuleSet::STARTING_CHIP_OPTIONS
        .iter()
        .position(|chips| *chips == rules_value.starting_chips)
        .unwrap_or(2);
    let previous = chip_index.checked_sub(1).map(|index| TexasHoldemRuleSet {
        starting_chips: TexasHoldemRuleSet::STARTING_CHIP_OPTIONS[index],
        ..rules_value
    });
    let next = TexasHoldemRuleSet::STARTING_CHIP_OPTIONS
        .get(chip_index + 1)
        .copied()
        .map(|starting_chips| TexasHoldemRuleSet {
            starting_chips,
            ..rules_value
        });
    add_texas_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "初始筹码",
            value: rules_value.starting_chips.to_string(),
            help: "每位玩家入桌时拥有的筹码。大盲固定为 2，小盲固定为 1。",
            editable: can_configure,
            previous: previous.filter(|_| can_configure),
            next: next.filter(|_| can_configure),
        },
        assets,
    );
    let toggled = TexasHoldemRuleSet {
        short_deck: !rules_value.short_deck,
        ..rules_value
    };
    add_texas_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "奥马哈",
            value: if rules_value.omaha {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后每人发四张底牌；最终牌型必须恰好使用两张底牌和三张公共牌。",
            editable: can_configure,
            previous: can_configure.then_some(TexasHoldemRuleSet {
                omaha: !rules_value.omaha,
                ..rules_value
            }),
            next: can_configure.then_some(TexasHoldemRuleSet {
                omaha: !rules_value.omaha,
                ..rules_value
            }),
        },
        assets,
    );
    add_texas_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "短牌模式",
            value: if rules_value.short_deck {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后移除 2、3、4、5。短牌中同花高于葫芦，三条高于顺子。",
            editable: can_configure,
            previous: can_configure.then_some(toggled),
            next: can_configure.then_some(toggled),
        },
        assets,
    );
    let ignore_kickers_toggled = TexasHoldemRuleSet {
        ignore_kickers: !rules_value.ignore_kickers,
        ..rules_value
    };
    add_texas_rule_config_row(
        commands,
        rules_panel,
        TexasRuleConfigRow {
            label: "只比较最大牌型",
            value: if rules_value.ignore_kickers {
                "开启".to_owned()
            } else {
                "关闭".to_owned()
            },
            help: "开启后忽略踢脚牌；高牌和同花只比较最大的一张牌。",
            editable: can_configure,
            previous: can_configure.then_some(ignore_kickers_toggled),
            next: can_configure.then_some(ignore_kickers_toggled),
        },
        assets,
    );

    let players = add_panel(
        commands,
        content,
        Node {
            min_width: px(380),
            flex_basis: px(560),
            flex_grow: 2.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            ..default()
        },
        PANEL_ALT,
        PanelSkin::Section,
        assets,
    );
    add_section_title(
        commands,
        players,
        format!("玩家席位  {connected_count}/{TABLE_SEAT_COUNT}"),
        assets,
    );
    render_seat_selector(commands, players, client, lobby, assets, avatars);

    let actions = spawn_node(
        commands,
        players,
        Node {
            width: percent(100),
            min_height: px(48),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(12),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            ..default()
        },
        None,
    );
    commands
        .entity(actions)
        .insert((GlobalZIndex(800), FocusPolicy::Pass));
    let you = client.0.model().you();
    let ready = you
        .and_then(|you| lobby.players.iter().find(|player| player.id == you))
        .is_some_and(|player| player.ready);
    let is_host = you == lobby.host;
    add_action_button(
        commands,
        actions,
        "退出房间",
        UiAction::LeaveRoom,
        ButtonKind::Pass,
        assets,
    );
    if is_host {
        let can_start = connected_count >= usize::from(TexasHoldemRuleSet::MIN_PLAYERS)
            && lobby
                .players
                .iter()
                .filter(|player| player.connected)
                .all(|player| player.seat.is_some() && player.ready);
        if can_start {
            add_action_button(
                commands,
                actions,
                "开始游戏",
                UiAction::StartGame,
                ButtonKind::Primary,
                assets,
            );
        } else {
            add_disabled_action_button(commands, actions, "等待玩家中", assets);
        }
    } else {
        add_action_button(
            commands,
            actions,
            if ready { "取消准备" } else { "准备" },
            UiAction::ToggleReady,
            if ready {
                ButtonKind::Secondary
            } else {
                ButtonKind::Primary
            },
            assets,
        );
    }
}

fn render_seat_selector(
    commands: &mut Commands,
    parent: Entity,
    client: &ClientResource,
    lobby: &leocard_protocol::LobbySnapshot,
    assets: &UiAssets,
    avatars: &AvatarImages,
) {
    let ring = spawn_node(
        commands,
        parent,
        Node {
            width: px(620),
            height: px(390),
            max_width: percent(100),
            align_self: AlignSelf::Center,
            position_type: PositionType::Relative,
            ..default()
        },
        None,
    );
    let table = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(150),
                top: px(108),
                width: px(320),
                height: px(174),
                padding: UiRect::axes(px(24), px(18)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(percent(50)),
                overflow: Overflow::clip(),
                ..default()
            },
            ImageNode::new(assets.table_felt.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgba(0.72, 0.83, 0.76, 0.92)),
            BackgroundColor(TABLE_BG),
            BorderColor::all(Color::srgba(0.62, 0.82, 0.70, 0.28)),
            BoxShadow::new(Color::BLACK.with_alpha(0.38), px(2), px(7), px(0), px(9)),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(ring).add_child(table);
    let connected_count = connected_lobby_player_count(lobby);
    let ready_count = lobby
        .players
        .iter()
        .filter(|player| player.connected && player.ready)
        .count();
    add_text(
        commands,
        table,
        format!("等待准备  {ready_count}/{connected_count}"),
        17.0,
        TEXT,
        assets,
    );
    #[cfg(feature = "developer")]
    if client.0.model().you() == lobby.host {
        add_text(
            commands,
            table,
            "右键空座添加机器人，右键机器人移除",
            10.5,
            MUTED,
            assets,
        );
    }
    let rule_chips = spawn_node(
        commands,
        table,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            column_gap: px(6),
            ..default()
        },
        None,
    );
    let rule_labels = match &lobby.rules {
        leocard_protocol::GameRules::QiGui523(rules) => vec![
            format!("{}副", rules.deck_count),
            format!("{}张", rules.hand_size),
            time_control_label(rules.time_control).to_owned(),
        ],
        leocard_protocol::GameRules::TexasHoldem(rules) => vec![
            format!("{}筹码", rules.starting_chips),
            match (rules.omaha, rules.short_deck) {
                (true, true) => "短牌奥马哈".to_owned(),
                (true, false) => "奥马哈".to_owned(),
                (false, true) => "短牌德州".to_owned(),
                (false, false) => "标准德州".to_owned(),
            },
            if rules.ignore_kickers {
                "只比较最大牌型".to_owned()
            } else {
                "标准比牌".to_owned()
            },
        ],
        leocard_protocol::GameRules::Shengji(rules) => vec![
            format!("{}副牌", rules.deck_count),
            if rules.bottom_copy {
                "允许抄底".to_owned()
            } else {
                "不抄底".to_owned()
            },
            if rules.five_trump_crossing {
                "五主过江".to_owned()
            } else {
                "不过江".to_owned()
            },
        ],
        leocard_protocol::GameRules::Uno(rules) if rules.is_no_mercy() => vec![
            "No Mercy".to_owned(),
            "+2 至 +10 递增堆叠".to_owned(),
            if rules.no_mercy.mercy_elimination {
                "25 张淘汰".to_owned()
            } else {
                "不启用慈悲淘汰".to_owned()
            },
        ],
        leocard_protocol::GameRules::Uno(rules) if rules.is_flip() => vec![
            "UNO FLIP".to_owned(),
            if rules.flip.random_pairing {
                "随机双面配对".to_owned()
            } else {
                "固定双面配对".to_owned()
            },
            if rules.flip.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
        ],
        leocard_protocol::GameRules::Uno(rules) => vec![
            if rules.action_stacking {
                "功能牌可堆叠".to_owned()
            } else {
                "功能牌不堆叠".to_owned()
            },
            if rules.jump_in {
                "允许抢出".to_owned()
            } else {
                "不抢出".to_owned()
            },
            if rules.uno_callout {
                "UNO 检举".to_owned()
            } else {
                "不检举".to_owned()
            },
        ],
    };
    for label in rule_labels {
        add_lobby_rule_chip(commands, rule_chips, label, assets);
    }

    let you = client.0.model().you();
    let seat_count = match &lobby.rules {
        leocard_protocol::GameRules::Shengji(_) => ShengjiRuleSet::PLAYER_COUNT as u8,
        leocard_protocol::GameRules::Uno(_) => UnoRuleSet::MAX_PLAYERS,
        leocard_protocol::GameRules::QiGui523(_) | leocard_protocol::GameRules::TexasHoldem(_) => {
            TABLE_SEAT_COUNT
        }
    };
    for seat_index in 0..seat_count {
        let seat = SeatId(seat_index);
        let occupant = lobby
            .players
            .iter()
            .find(|player| player.seat == Some(seat));
        let is_you = occupant.is_some_and(|player| Some(player.id) == you);
        let is_host = occupant.is_some_and(|player| Some(player.id) == lobby.host);
        let (left, top) = if lobby.game == GameKind::Shengji {
            match seat_index {
                0 => (239.0, 290.0),
                1 => (0.0, 145.0),
                2 => (239.0, 0.0),
                3 => (478.0, 145.0),
                _ => unreachable!("双升固定四个座位"),
            }
        } else {
            lobby_seat_position(seat_index)
        };
        let entity = commands
            .spawn((
                Button,
                UiAction::SelectSeat(seat),
                LobbySeatHover {
                    seat: seat_index,
                    amount: 0.0,
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(left),
                    top: px(top),
                    width: px(142),
                    height: px(100),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::NONE),
                UiTransform::IDENTITY,
            ))
            .id();
        commands.entity(ring).add_child(entity);
        let visual = spawn_node(
            commands,
            entity,
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(3),
                ..default()
            },
            None,
        );
        commands.entity(visual).insert((
            LobbySeatVisual(seat_index),
            UiTransform::IDENTITY,
            FocusPolicy::Pass,
        ));
        if let Some(player) = occupant {
            commands
                .entity(entity)
                .insert(LobbySeatTransitionSource(player.id));
            let avatar_ring = spawn_node(
                commands,
                visual,
                Node {
                    width: px(60),
                    height: px(60),
                    min_width: px(60),
                    position_type: PositionType::Relative,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(if is_you { 3 } else { 2 })),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::srgba(0.02, 0.08, 0.06, 0.80)),
            );
            commands.entity(avatar_ring).insert((
                BorderColor::all(if is_you {
                    ACCENT
                } else if player.ready {
                    READY
                } else {
                    MUTED.with_alpha(0.42)
                }),
                BoxShadow::new(
                    if is_you {
                        ACCENT.with_alpha(0.24)
                    } else {
                        Color::BLACK.with_alpha(0.24)
                    },
                    px(0),
                    px(2),
                    px(0),
                    px(5),
                ),
            ));
            let handle = player.avatar.and_then(|id| avatars.remote.get(&id));
            let avatar = add_avatar(commands, avatar_ring, &player.name, handle, 52.0, assets);
            if is_host {
                add_host_crown(commands, avatar, assets);
            }
            if player.ready {
                let check = spawn_node(
                    commands,
                    avatar_ring,
                    Node {
                        position_type: PositionType::Absolute,
                        right: px(-2),
                        bottom: px(-1),
                        width: px(18),
                        height: px(18),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    Some(READY),
                );
                add_text(commands, check, "✓", 11.5, Color::WHITE, assets);
            }
            add_text(
                commands,
                visual,
                &player.name,
                14.0,
                if is_you { ACCENT } else { TEXT },
                assets,
            );
            let status = spawn_node(
                commands,
                visual,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: px(5),
                    ..default()
                },
                None,
            );
            add_text(
                commands,
                status,
                reference_level(player.reference_points),
                10.5,
                MUTED,
                assets,
            );
            add_text(
                commands,
                status,
                if player.ready {
                    "已准备"
                } else {
                    "未准备"
                },
                12.5,
                if player.ready { READY } else { MUTED },
                assets,
            );
        } else {
            let empty_ring = spawn_node(
                commands,
                visual,
                Node {
                    width: px(56),
                    height: px(56),
                    position_type: PositionType::Relative,
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(percent(50)),
                    ..default()
                },
                Some(Color::srgba(0.03, 0.12, 0.085, 0.64)),
            );
            commands.entity(empty_ring).insert((
                LobbyEmptySeatRing(seat_index),
                BorderColor::all(MUTED.with_alpha(0.34)),
            ));
            for node in [
                Node {
                    position_type: PositionType::Absolute,
                    left: px(15),
                    top: px(25),
                    width: px(22),
                    height: px(2),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(25),
                    top: px(15),
                    width: px(2),
                    height: px(22),
                    ..default()
                },
            ] {
                let stroke = spawn_node(commands, empty_ring, node, Some(MUTED.with_alpha(0.72)));
                commands.entity(stroke).insert(FocusPolicy::Pass);
            }
            let label = add_text(commands, visual, "空位", 12.0, MUTED, assets);
            commands
                .entity(label)
                .insert(LobbyEmptySeatLabel(seat_index));
        }
    }
}

fn connected_lobby_player_count(lobby: &leocard_protocol::LobbySnapshot) -> usize {
    lobby
        .players
        .iter()
        .filter(|player| player.connected)
        .count()
}

fn add_lobby_rule_chip(
    commands: &mut Commands,
    parent: Entity,
    label: impl Into<String>,
    assets: &UiAssets,
) {
    let chip = spawn_node(
        commands,
        parent,
        Node {
            min_height: px(25),
            padding: UiRect::axes(px(8), px(3)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(12)),
            ..default()
        },
        Some(Color::srgba(0.015, 0.065, 0.048, 0.72)),
    );
    commands
        .entity(chip)
        .insert(BorderColor::all(Color::srgba(0.70, 0.88, 0.78, 0.18)));
    add_text(commands, chip, label, 11.5, TEXT, assets);
}
