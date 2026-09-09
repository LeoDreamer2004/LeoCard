use super::super::*;
use crate::app::presentation::{
    ACCENT, BORDER, ButtonTint, HEADER_BG, MUTED, PANEL, TEXT, add_text, spawn_node,
};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, UiAction};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;

pub(super) fn spawn_chat_panel(
    commands: &mut Commands,
    parent: Entity,
    chat: &ChatPanelState,
) -> Entity {
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
    panel
}

pub(super) fn add_chat_history(
    commands: &mut Commands,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
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
}

pub(super) fn add_chat_input_row(
    commands: &mut Commands,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
    let row = spawn_node(
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
    add_text_input(commands, row, chat, assets);
    add_input_icon_button(
        commands,
        row,
        UiAction::Chat(ChatUiAction::ToggleEmojiMenu),
        &assets.social.chat_emoji_icon,
        24.0,
        assets,
    );
    add_input_icon_button(
        commands,
        row,
        UiAction::Chat(ChatUiAction::ToggleQuickVoiceMenu),
        &assets.social.quick_voice_icon,
        23.0,
        assets,
    );
}

fn add_text_input(commands: &mut Commands, row: Entity, chat: &ChatPanelState, assets: &UiAssets) {
    let button = commands
        .spawn((
            Button,
            UiAction::Chat(ChatUiAction::FocusInput),
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
    commands.entity(row).add_child(button);
    let input_display = if chat.input.is_empty() {
        "输入消息，回车发送".to_owned()
    } else {
        chat_input_display(&chat.input)
    };
    let label = add_text(
        commands,
        button,
        input_display,
        12.5,
        if chat.input.is_empty() { MUTED } else { TEXT },
        assets,
    );
    commands.entity(label).insert(ChatInputText);
}

fn add_input_icon_button(
    commands: &mut Commands,
    row: Entity,
    action: UiAction,
    icon: &Handle<Image>,
    icon_size: f32,
    assets: &UiAssets,
) {
    let button = commands
        .spawn((
            Button,
            action,
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
    commands.entity(row).add_child(button);
    let icon = commands
        .spawn((
            Node {
                width: px(icon_size),
                height: px(icon_size),
                ..default()
            },
            ImageNode::new(icon.clone()),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(icon);
}
