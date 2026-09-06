//! 玩家档案弹窗、游戏标签和互动统计视图。

use super::*;

pub fn add_profile_avatar_button(
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

pub fn render_profile_modal(
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
            ImageNode::new(assets.controls.secondary_button.clone())
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
                        .social
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
