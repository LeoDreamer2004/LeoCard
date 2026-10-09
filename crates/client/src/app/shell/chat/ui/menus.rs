use super::super::*;
use super::panel::{add_chat_button_highlight, cozy_chat_button_image};
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::UiAssets;
use crate::app::shell::{ChatUiAction, UiAction, add_cozy_panel};
use bevy::ecs::query::QueryFilter;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Button;
use leocard_protocol::ChatEmoji;

pub(super) fn add_quick_voice_menu(
    commands: &mut Commands,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
    let menu = add_cozy_panel(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            bottom: px(50),
            width: px(250),
            min_width: px(250),
            max_width: px(250),
            height: px(252),
            min_height: px(252),
            max_height: px(252),
            flex_shrink: 0.0,
            padding: UiRect::all(px(10)),
            ..default()
        },
        assets,
    );
    commands.entity(menu).insert((
        QuickVoiceMenu,
        menu_visibility(chat.open && chat.quick_voice_open),
        GlobalZIndex(1810),
    ));
    let scroll = spawn_node(
        commands,
        menu,
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
    commands.entity(scroll).insert((
        QuickVoiceScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::new(0.0, chat.quick_voice_scroll_y)),
    ));
    for (index, voice) in QUICK_VOICES.iter().enumerate() {
        let button = commands
            .spawn((
                Button,
                UiAction::Chat(ChatUiAction::SendQuickVoice(index as u8)),
                Node {
                    width: px(222),
                    min_width: px(222),
                    max_width: px(222),
                    height: px(25),
                    min_height: px(25),
                    max_height: px(25),
                    flex_shrink: 0.0,
                    margin: UiRect::left(px(3.5)),
                    padding: UiRect::left(px(11)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                cozy_chat_button_image(assets.home.button.clone()),
            ))
            .id();
        commands.entity(scroll).add_child(button);
        add_chat_button_highlight(commands, button, assets);
        add_text(commands, button, *voice, 11.0, TEXT, assets);
    }
}

pub(super) fn add_emoji_menu(
    commands: &mut Commands,
    panel: Entity,
    chat: &ChatPanelState,
    assets: &UiAssets,
) {
    let menu = add_cozy_panel(
        commands,
        panel,
        Node {
            position_type: PositionType::Absolute,
            right: px(14),
            bottom: px(50),
            width: px(250),
            min_width: px(250),
            max_width: px(250),
            height: px(178),
            min_height: px(178),
            max_height: px(178),
            padding: UiRect::all(px(10)),
            ..default()
        },
        assets,
    );
    commands.entity(menu).insert((
        EmojiMenu,
        menu_visibility(chat.open && chat.emoji_open),
        GlobalZIndex(1810),
    ));
    let scroll = spawn_node(
        commands,
        menu,
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
    commands.entity(scroll).insert((
        EmojiScroll,
        RelativeCursorPosition::default(),
        ScrollPosition(Vec2::new(0.0, chat.emoji_scroll_y)),
    ));
    for emoji in ChatEmoji::ALL {
        let button = commands
            .spawn((
                Button,
                UiAction::Chat(ChatUiAction::SendEmoji(emoji)),
                Node {
                    width: px(42),
                    min_width: px(42),
                    max_width: px(42),
                    height: px(42),
                    min_height: px(42),
                    max_height: px(42),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                cozy_chat_button_image(assets.home.button.clone()),
            ))
            .id();
        commands.entity(scroll).add_child(button);
        add_chat_button_highlight(commands, button, assets);
        let icon = commands
            .spawn((
                Node {
                    width: px(38),
                    height: px(38),
                    ..default()
                },
                ImageNode::new(assets.chat_emoji(emoji)),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(button).add_child(icon);
    }
}

fn menu_visibility(open: bool) -> Visibility {
    if open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

#[expect(
    clippy::type_complexity,
    reason = "separate filtered Bevy queries prevent the two menus from overlapping"
)]
pub(crate) fn scroll_chat_menus(
    mut wheels: MessageReader<MouseWheel>,
    mut chat: ResMut<ChatPanelState>,
    mut quick_voice_scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        (With<QuickVoiceScroll>, Without<EmojiScroll>),
    >,
    mut emoji_scrolls: Query<
        (&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode),
        (With<EmojiScroll>, Without<QuickVoiceScroll>),
    >,
) {
    let delta = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 28.0,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum::<f32>();
    if delta == 0.0 || !chat.open {
        return;
    }
    if chat.quick_voice_open {
        scroll_hovered_menu(
            delta,
            &mut quick_voice_scrolls,
            &mut chat.quick_voice_scroll_y,
        );
    }
    if chat.emoji_open {
        scroll_hovered_menu(delta, &mut emoji_scrolls, &mut chat.emoji_scroll_y);
    }
}

fn scroll_hovered_menu<F: QueryFilter>(
    delta: f32,
    scrolls: &mut Query<(&RelativeCursorPosition, &mut ScrollPosition, &ComputedNode), F>,
    saved_position: &mut f32,
) {
    for (cursor, mut position, node) in scrolls {
        if !cursor.cursor_over() {
            continue;
        }
        let maximum =
            ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
        let next = (position.y - delta).clamp(0.0, maximum);
        position.y = next;
        *saved_position = next;
    }
}
