use super::super::*;
use crate::app::presentation::{MUTED, TEXT};
use bevy::ecs::query::QueryFilter;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub(crate) fn sync_chat_panel_text(
    chat: Res<ChatPanelState>,
    mut history_texts: Query<&mut Text, (With<ChatHistoryText>, Without<ChatInputText>)>,
    mut input_texts: Query<(&mut Text, &mut TextColor), With<ChatInputText>>,
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
    let (input, color) = if chat.input.is_empty() {
        ("输入消息，回车发送".to_owned(), MUTED)
    } else {
        (chat_input_display(&chat.input), TEXT)
    };
    for (mut text, mut text_color) in &mut input_texts {
        if text.0 != input {
            text.0.clone_from(&input);
        }
        if text_color.0 != color {
            text_color.0 = color;
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

pub(crate) fn chat_input_display(input: &str) -> String {
    const VISIBLE_CHARS: usize = 25;
    let count = input.chars().count();
    if count <= VISIBLE_CHARS {
        return input.to_owned();
    }
    let tail = input
        .chars()
        .skip(count - VISIBLE_CHARS)
        .collect::<String>();
    format!("…{tail}")
}

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
