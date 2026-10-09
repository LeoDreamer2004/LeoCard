use super::super::*;
use super::controls::{add_auto_play_toggle, add_auxiliary_actions, add_chat_toggle};
use super::menus::{add_emoji_menu, add_quick_voice_menu};
use crate::app::presentation::ButtonHighlight;
use crate::app::presentation::{TEXT, TextInput, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, UiAction, add_cozy_panel};
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, VisualBox};
use leocard_protocol::MAX_CHAT_MESSAGE_CHARS;

pub(crate) struct ChatAuxiliaryAction {
    pub label: &'static str,
    pub action: Option<UiAction>,
    pub highlighted: bool,
}

pub(crate) fn add_chat_panel(
    commands: &mut Commands,
    parent: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
    auto_play: Option<bool>,
    auxiliary_actions: &[ChatAuxiliaryAction],
) -> Entity {
    let panel = spawn_chat_panel(commands, parent, chat, assets);
    add_chat_toggle(commands, panel, chat, assets);
    if let Some(enabled) = auto_play {
        add_auto_play_toggle(commands, panel, enabled, assets);
    }
    add_auxiliary_actions(commands, panel, auxiliary_actions, assets);
    add_chat_history(commands, panel, chat, assets);
    add_chat_input_row(commands, panel, chat, assets);
    add_quick_voice_menu(commands, panel, chat, assets);
    add_emoji_menu(commands, panel, chat, assets);
    panel
}

pub(super) fn spawn_chat_panel(
    commands: &mut Commands,
    parent: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) -> Entity {
    let panel = add_cozy_panel(
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
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            ..default()
        },
        assets,
    );
    commands.entity(panel).insert((
        ChatPanel,
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
    add_text(commands, panel, "聊天", 17.0, TEXT, assets);
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
            padding: UiRect::all(px(10)),
            overflow: Overflow::clip(),
            ..default()
        },
        None,
    );
    commands
        .entity(history_box)
        .insert(cozy_chat_history_image(assets));
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
            column_gap: px(4),
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
    let mut input = TextInput::new("chat.message", &chat.input);
    input.placeholder = "输入消息，回车发送";
    input.max_characters = MAX_CHAT_MESSAGE_CHARS;
    input.font_size = 12.5;
    let editor = input.spawn(
        commands,
        row,
        Node {
            width: px(CHAT_PANEL_WIDTH - 124.0),
            min_width: px(CHAT_PANEL_WIDTH - 124.0),
            max_width: px(CHAT_PANEL_WIDTH - 124.0),
            height: percent(100),
            flex_shrink: 0.0,
            padding: UiRect::axes(px(9), px(5)),
            ..default()
        },
        assets,
    );
    commands.entity(editor).insert(ChatTextInput);
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
                ..default()
            },
            cozy_chat_button_image(assets.home.button.clone()),
        ))
        .id();
    commands.entity(row).add_child(button);
    add_chat_button_highlight(commands, button, assets);
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

pub(super) fn cozy_chat_button_image(texture: Handle<Image>) -> ImageNode {
    let mut image = ImageNode::new(texture).with_mode(NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(32.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 0.40,
    }));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn cozy_chat_history_image(assets: &UiAssets) -> ImageNode {
    let mut image =
        ImageNode::new(assets.home.input.clone()).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(32.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 0.45,
        }));
    image.visual_box = VisualBox::BorderBox;
    image
}

pub(super) fn add_chat_button_highlight(
    commands: &mut Commands,
    button: Entity,
    assets: &UiAssets,
) {
    let overlay = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                ..default()
            },
            cozy_chat_button_image(assets.home.purple_button_compact.clone()),
            Visibility::Hidden,
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(button).add_child(overlay);
    commands.entity(button).insert(ButtonHighlight {
        overlay,
        arrows: None,
    });
}

pub(crate) fn sync_chat_panel_text(
    chat: Res<ChatPanelState>,
    mut history_texts: Query<&mut Text, With<ChatHistoryText>>,
    mut quick_menus: Query<&mut Visibility, (With<QuickVoiceMenu>, Without<EmojiMenu>)>,
    mut emoji_menus: Query<&mut Visibility, (With<EmojiMenu>, Without<QuickVoiceMenu>)>,
) {
    if !chat.is_changed() {
        return;
    }
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
    for mut text in &mut history_texts {
        if text.0 != history {
            text.0.clone_from(&history);
        }
    }
    for mut visibility in &mut quick_menus {
        let expected = if chat.open && chat.quick_voice_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
    for mut visibility in &mut emoji_menus {
        let expected = if chat.open && chat.emoji_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != expected {
            *visibility = expected;
        }
    }
}
