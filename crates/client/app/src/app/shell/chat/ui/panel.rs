use super::super::*;
use super::controls::{add_auto_play_toggle, add_auxiliary_actions, add_chat_toggle};
use super::menus::{add_emoji_menu, add_quick_voice_menu};
use crate::app::presentation::{GAME_SIDE_CONTROL_SCALE, MUTED, TEXT, add_text};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, UiAction, add_cozy_panel};
use bevy::prelude::*;
use leocard_protocol::MAX_CHAT_MESSAGE_CHARS;
use leocard_ui::{DrawerSlide, TextInput, TextureButton, UiComposer, sliced_image, spawn_node};

pub(crate) struct ChatAuxiliaryAction {
    pub label: &'static str,
    pub action: Option<UiAction>,
    pub highlighted: bool,
}

/// Returns the side-control anchor, including its slide and platform-specific scale.
pub(crate) fn add_chat_panel(
    commands: &mut UiComposer,
    parent: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
    auto_play: Option<bool>,
    auxiliary_actions: &[ChatAuxiliaryAction],
) -> Entity {
    let panel = spawn_chat_panel(commands, parent, chat, assets);
    let controls = spawn_side_controls(commands, panel);
    add_chat_toggle(commands, controls, chat, assets);
    if let Some(enabled) = auto_play {
        add_auto_play_toggle(commands, controls, enabled, assets);
    }
    add_auxiliary_actions(commands, controls, auxiliary_actions, assets);
    add_chat_history(commands, panel, chat, assets);
    if chat.read_only {
        add_text(commands, panel, "观战中，仅查看聊天", 13.0, MUTED, assets);
    } else {
        add_chat_input_row(commands, panel, chat, assets);
        add_quick_voice_menu(commands, panel, chat, assets);
        add_emoji_menu(commands, panel, chat, assets);
    }
    controls
}

fn spawn_side_controls(commands: &mut UiComposer, panel: Entity) -> Entity {
    let anchor = spawn_node(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(0),
            width: px(0),
            height: px(0),
            ..default()
        },
        None,
    );
    commands
        .entity(anchor)
        .insert(UiTransform::from_scale(Vec2::splat(
            GAME_SIDE_CONTROL_SCALE,
        )));
    spawn_node(
        commands,
        anchor,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            bottom: px(0),
            width: px(0),
            height: px(CHAT_PANEL_HEIGHT),
            ..default()
        },
        None,
    )
}

pub(super) fn spawn_chat_panel(
    commands: &mut UiComposer,
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
            height: px(CHAT_PANEL_HEIGHT),
            min_height: px(CHAT_PANEL_HEIGHT),
            max_height: px(CHAT_PANEL_HEIGHT),
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
            translation: chat.drawer.translation(),
            ..UiTransform::IDENTITY
        },
        GlobalZIndex(1800),
    ));
    commands
        .entity(panel)
        .entry::<DrawerSlide>()
        .or_insert(chat.drawer.clone());
    panel
}

pub(super) fn add_chat_history(
    commands: &mut UiComposer,
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
    let history = chat.history_text();
    let history_text = add_text(commands, history_box, history, 12.5, TEXT, assets);
    commands.entity(history_text).insert(ChatHistoryText);
}

pub(super) fn add_chat_input_row(
    commands: &mut UiComposer,
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

fn add_text_input(
    commands: &mut UiComposer,
    row: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
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
        assets.text_input_style(),
    );
    commands.entity(editor).insert(ChatTextInput);
}

fn add_input_icon_button(
    commands: &mut UiComposer,
    row: Entity,
    action: UiAction,
    icon: &Handle<Image>,
    icon_size: f32,
    assets: &UiAssets,
) {
    TextureButton {
        node: Node {
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
        normal: cozy_chat_button_image(assets.home.button.clone()),
        hovered: cozy_chat_button_image(assets.home.purple_button_compact.clone()),
        action: Some(action),
        label: None,
        font_size: 13.0,
        color: TEXT,
        icon: Some((icon.clone(), icon_size)),
    }
    .mount(commands, row, &assets.font);
}

pub(super) fn cozy_chat_button_image(texture: Handle<Image>) -> ImageNode {
    sliced_image(texture, 32.0, 0.40)
}

pub(super) fn cozy_chat_history_image(assets: &UiAssets) -> ImageNode {
    sliced_image(assets.home.input.clone(), 32.0, 0.45)
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
    let history = chat.history_text();
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
