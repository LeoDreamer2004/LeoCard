//! In-game chat drawer, quick voices, and developer hand editor.

use super::*;

pub fn add_chat_panel(
    commands: &mut Commands,
    parent: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
    auto_play: Option<bool>,
    previous_trick_available: Option<bool>,
    buried_cards_available: Option<bool>,
) {
    let panel = spawn_node(
        commands,
        parent,
        Node {
            position_type: PositionType::Absolute,
            right: px(0),
            bottom: px(8),
            width: px(CHAT_PANEL_WIDTH),
            min_width: px(CHAT_PANEL_WIDTH),
            max_width: px(CHAT_PANEL_WIDTH),
            height: px(340),
            min_height: px(340),
            max_height: px(340),
            flex_shrink: 0.0,
            padding: UiRect::all(px(8)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.97)),
    );
    commands.entity(panel).insert((
        ChatPanel,
        BorderColor::all(ACCENT.with_alpha(0.55)),
        UiTransform {
            translation: Val2::px(CHAT_PANEL_HIDDEN_OFFSET * chat.slide, 0.0),
            ..UiTransform::IDENTITY
        },
        GlobalZIndex(1800),
    ));
    let toggle = commands
        .spawn((
            Button,
            UiAction::ToggleChatPanel,
            ButtonTint {
                normal: Color::WHITE,
                hovered: Color::srgb(1.0, 0.90, 0.56),
                pressed: Color::srgb(0.68, 0.82, 0.90),
            },
            Node {
                position_type: PositionType::Absolute,
                left: px(-32),
                top: px(154),
                width: px(32),
                min_width: px(32),
                max_width: px(32),
                height: px(32),
                min_height: px(32),
                max_height: px(32),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(if chat.open {
                assets.social.chat_close_icon.clone()
            } else {
                assets.social.chat_open_icon.clone()
            })
            .with_color(Color::WHITE),
            ChatToggleIcon,
        ))
        .id();
    commands.entity(panel).add_child(toggle);

    if let Some(enabled) = auto_play {
        let auto_button = commands
            .spawn((
                Button,
                UiAction::ToggleAutoPlay,
                ButtonTint {
                    normal: if enabled {
                        Color::srgb(0.18, 0.68, 0.38)
                    } else {
                        Color::srgb(0.13, 0.39, 0.29)
                    },
                    hovered: if enabled {
                        Color::srgb(0.43, 0.94, 0.60)
                    } else {
                        Color::srgb(0.12, 0.38, 0.27)
                    },
                    pressed: Color::srgb(0.11, 0.27, 0.21),
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(-32),
                    // Keep the autoplay control below the chat drawer arrow.  The
                    // old value overlapped the 32 px arrow by 18 px.
                    top: px(194),
                    width: px(32),
                    min_width: px(32),
                    max_width: px(32),
                    height: px(32),
                    min_height: px(32),
                    max_height: px(32),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BorderColor::all(if enabled { ACCENT } else { BORDER }),
                ImageNode::new(assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(if enabled {
                        Color::srgb(0.18, 0.68, 0.38)
                    } else {
                        Color::srgb(0.13, 0.39, 0.29)
                    }),
            ))
            .id();
        commands.entity(panel).add_child(auto_button);
        let icon = commands
            .spawn((
                Node {
                    width: px(25),
                    height: px(25),
                    ..default()
                },
                ImageNode::new(assets.controls.robot_icon.clone()),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(auto_button).add_child(icon);
    }

    if let Some(available) = previous_trick_available {
        // 可回看时只启用交互，不用黄色边框打断牌桌视觉。
        let normal = Color::srgb(0.13, 0.39, 0.29);
        let previous_button = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(-32),
                    top: px(234),
                    width: px(32),
                    min_width: px(32),
                    max_width: px(32),
                    height: px(32),
                    min_height: px(32),
                    max_height: px(32),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BorderColor::all(BORDER),
                ImageNode::new(assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id();
        if available {
            commands.entity(previous_button).insert((
                Button,
                UiAction::ShowShengjiPreviousTrick,
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.22, 0.56, 0.39),
                    pressed: Color::srgb(0.10, 0.27, 0.20),
                },
            ));
        }
        commands.entity(panel).add_child(previous_button);
        let label = add_text(commands, previous_button, "上轮", 10.0, MUTED, assets);
        commands.entity(label).insert(FocusPolicy::Pass);
    }

    if let Some(available) = buried_cards_available {
        let normal = if available {
            Color::srgb(0.13, 0.39, 0.29)
        } else {
            Color::srgb(0.09, 0.16, 0.14)
        };
        let buried_button = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(-32),
                    top: px(274),
                    width: px(32),
                    min_width: px(32),
                    max_width: px(32),
                    height: px(32),
                    min_height: px(32),
                    max_height: px(32),
                    flex_shrink: 0.0,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BorderColor::all(if available { ACCENT } else { BORDER }),
                ImageNode::new(assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(normal),
            ))
            .id();
        if available {
            commands.entity(buried_button).insert((
                Button,
                UiAction::ToggleShengjiBuried,
                ButtonTint {
                    normal,
                    hovered: Color::srgb(0.22, 0.56, 0.39),
                    pressed: Color::srgb(0.10, 0.27, 0.20),
                },
            ));
        }
        commands.entity(panel).add_child(buried_button);
        let label = add_text(
            commands,
            buried_button,
            "底牌",
            10.0,
            if available { ACCENT } else { MUTED },
            assets,
        );
        commands.entity(label).insert(FocusPolicy::Pass);
    }

    add_text(commands, panel, "聊天", 15.0, ACCENT, assets);
    let history_box = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            min_width: percent(100),
            max_width: percent(100),
            height: px(0),
            min_height: px(0),
            max_height: px(1000),
            flex_grow: 1.0,
            flex_shrink: 1.0,
            padding: UiRect::all(px(7)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            overflow: Overflow::clip(),
            ..default()
        },
        Some(PANEL.with_alpha(0.78)),
    );
    commands
        .entity(history_box)
        .insert(BorderColor::all(BORDER));
    let history = chat
        .history
        .iter()
        .rev()
        .take(10)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|entry| format!("[{}(玩家)]: {}", entry.player_name, entry.message))
        .collect::<Vec<_>>()
        .join("\n");
    let history_text = add_text(commands, history_box, history, 12.5, TEXT, assets);
    commands.entity(history_text).insert(ChatHistoryText);

    let input_row = spawn_node(
        commands,
        panel,
        Node {
            width: percent(100),
            min_width: percent(100),
            max_width: percent(100),
            height: px(38),
            min_height: px(38),
            max_height: px(38),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Row,
            column_gap: px(0),
            ..default()
        },
        None,
    );
    let input_button = commands
        .spawn((
            Button,
            UiAction::FocusChatInput,
            ButtonTint {
                normal: Color::srgb(0.08, 0.18, 0.15),
                hovered: Color::srgb(0.11, 0.26, 0.20),
                pressed: Color::srgb(0.06, 0.14, 0.12),
            },
            Node {
                width: px(CHAT_PANEL_WIDTH - 100.0),
                min_width: px(CHAT_PANEL_WIDTH - 100.0),
                max_width: px(CHAT_PANEL_WIDTH - 100.0),
                height: percent(100),
                min_height: percent(100),
                max_height: percent(100),
                flex_grow: 0.0,
                flex_shrink: 0.0,
                padding: UiRect::axes(px(9), px(5)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(7)),
                overflow: Overflow::clip_x(),
                ..default()
            },
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.08, 0.18, 0.15)),
            BorderColor::all(if chat.focused { ACCENT } else { BORDER }),
        ))
        .id();
    commands.entity(input_row).add_child(input_button);
    let input_display = if chat.input.is_empty() {
        "输入消息，回车发送".to_owned()
    } else {
        chat_input_display(&chat.input)
    };
    let input_label = add_text(
        commands,
        input_button,
        input_display,
        12.5,
        if chat.input.is_empty() { MUTED } else { TEXT },
        assets,
    );
    commands.entity(input_label).insert(ChatInputText);

    let emoji_button = commands
        .spawn((
            Button,
            UiAction::ToggleEmojiMenu,
            ButtonTint {
                normal: Color::srgb(0.20, 0.45, 0.35),
                hovered: Color::srgb(0.28, 0.60, 0.45),
                pressed: Color::srgb(0.13, 0.33, 0.26),
            },
            Node {
                width: px(42),
                min_width: px(42),
                max_width: px(42),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.20, 0.45, 0.35)),
        ))
        .id();
    commands.entity(input_row).add_child(emoji_button);
    let emoji_icon = commands
        .spawn((
            Node {
                width: px(24),
                height: px(24),
                ..default()
            },
            ImageNode::new(assets.social.chat_emoji_icon.clone()).with_color(Color::WHITE),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(emoji_button).add_child(emoji_icon);

    let voice_button = commands
        .spawn((
            Button,
            UiAction::ToggleQuickVoiceMenu,
            ButtonTint {
                normal: Color::srgb(0.20, 0.45, 0.35),
                hovered: Color::srgb(0.28, 0.60, 0.45),
                pressed: Color::srgb(0.13, 0.33, 0.26),
            },
            Node {
                width: px(42),
                min_width: px(42),
                max_width: px(42),
                height: percent(100),
                min_height: percent(100),
                max_height: percent(100),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            ImageNode::new(assets.controls.secondary_button.clone())
                .with_mode(NodeImageMode::Stretch)
                .with_color(Color::srgb(0.20, 0.45, 0.35)),
        ))
        .id();
    commands.entity(input_row).add_child(voice_button);
    let voice_icon = commands
        .spawn((
            Node {
                width: px(23),
                height: px(23),
                ..default()
            },
            ImageNode::new(assets.social.quick_voice_icon.clone()),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(voice_button).add_child(voice_icon);

    let quick_menu = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            right: px(8),
            bottom: px(50),
            width: px(285),
            min_width: px(285),
            max_width: px(285),
            height: px(252),
            min_height: px(252),
            max_height: px(252),
            flex_shrink: 0.0,
            padding: UiRect::all(px(2)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.99)),
    );
    commands.entity(quick_menu).insert((
        QuickVoiceMenu,
        BorderColor::all(ACCENT.with_alpha(0.72)),
        if chat.open && chat.quick_voice_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        GlobalZIndex(1810),
    ));
    let voice_scroll = spawn_node(
        commands,
        quick_menu,
        Node {
            width: percent(100),
            min_width: percent(100),
            max_width: percent(100),
            height: percent(100),
            min_height: percent(100),
            max_height: percent(100),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(voice_scroll).insert((
        QuickVoiceScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::new(0.0, chat.quick_voice_scroll_y)),
    ));
    for (index, voice) in QUICK_VOICES.iter().enumerate() {
        let button = commands
            .spawn((
                Button,
                UiAction::SendQuickVoice(index as u8),
                ButtonTint {
                    normal: Color::srgb(0.10, 0.26, 0.20),
                    hovered: Color::srgb(0.18, 0.43, 0.32),
                    pressed: Color::srgb(0.07, 0.19, 0.15),
                },
                Node {
                    width: px(273),
                    min_width: px(273),
                    max_width: px(273),
                    height: px(25),
                    min_height: px(25),
                    max_height: px(25),
                    flex_shrink: 0.0,
                    padding: UiRect::left(px(11)),
                    margin: UiRect::right(px(6)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                ImageNode::new(assets.controls.secondary_button.clone())
                    .with_mode(NodeImageMode::Stretch)
                    .with_color(Color::srgb(0.10, 0.26, 0.20)),
            ))
            .id();
        commands.entity(voice_scroll).add_child(button);
        add_text(commands, button, *voice, 11.0, TEXT, assets);
    }

    let emoji_menu = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            right: px(8),
            bottom: px(50),
            width: px(285),
            min_width: px(285),
            max_width: px(285),
            height: px(178),
            min_height: px(178),
            max_height: px(178),
            padding: UiRect::all(px(3)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        Some(HEADER_BG.with_alpha(0.99)),
    );
    commands.entity(emoji_menu).insert((
        EmojiMenu,
        BorderColor::all(ACCENT.with_alpha(0.72)),
        if chat.open && chat.emoji_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        GlobalZIndex(1810),
    ));
    let emoji_scroll = spawn_node(
        commands,
        emoji_menu,
        Node {
            width: percent(100),
            min_width: percent(100),
            max_width: percent(100),
            height: percent(100),
            min_height: percent(100),
            max_height: percent(100),
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            row_gap: px(2),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        None,
    );
    commands.entity(emoji_scroll).insert((
        EmojiScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::new(0.0, chat.emoji_scroll_y)),
    ));
    for emoji in ChatEmoji::ALL {
        let image = assets.chat_emoji(emoji);
        let button = commands
            .spawn((
                Button,
                UiAction::SendEmoji(emoji),
                ButtonTint {
                    normal: Color::srgb(0.11, 0.26, 0.20),
                    hovered: Color::srgb(0.18, 0.43, 0.32),
                    pressed: Color::srgb(0.07, 0.19, 0.15),
                },
                Node {
                    width: px(42),
                    min_width: px(42),
                    max_width: px(42),
                    height: px(42),
                    min_height: px(42),
                    max_height: px(42),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(px(7)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .id();
        commands.entity(emoji_scroll).add_child(button);
        let icon = commands
            .spawn((
                Node {
                    width: px(38),
                    height: px(38),
                    ..default()
                },
                ImageNode::new(image),
                FocusPolicy::Pass,
            ))
            .id();
        commands.entity(button).add_child(icon);
    }
}

#[cfg(feature = "developer")]
pub fn add_developer_hand_input(
    commands: &mut Commands,
    parent: Entity,
    input: &DeveloperHandInput,
    placeholder: &'static str,
    position: Vec2,
    assets: &UiAssets,
) {
    let field = commands
        .spawn((
            Button,
            UiAction::FocusDeveloperHand,
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                bottom: px(position.y),
                width: px(300),
                height: px(48),
                padding: UiRect::axes(px(12), px(7)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(if input.focused { 2 } else { 1 })),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BackgroundColor(HEADER_BG.with_alpha(0.94)),
            BorderColor::all(if input.focused { ACCENT } else { BORDER }),
            DeveloperHandInputField,
        ))
        .id();
    commands.entity(parent).add_child(field);
    let text = add_text(
        commands,
        field,
        developer_hand_input_label(input, placeholder),
        14.0,
        if input.value.is_empty() { MUTED } else { TEXT },
        assets,
    );
    commands
        .entity(text)
        .insert(DeveloperHandInputText { placeholder });
}
