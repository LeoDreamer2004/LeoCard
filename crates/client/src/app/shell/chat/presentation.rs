//! 聊天消息、表情气泡和抽屉动画。

use super::*;

const CHAT_HISTORY_LIMIT: usize = 60;

pub fn sync_chat_messages(
    mut commands: Commands,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    layers: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    avatars: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    existing_bubbles: Query<(Entity, &ActiveChatBubble)>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let messages = client.0.take_chat_messages();
    if messages.is_empty() {
        return;
    }
    let layer = layers.single().ok();
    let mut spawned = HashMap::<PlayerId, Entity>::new();
    for message in messages {
        let source = message.source;
        let player_name = client
            .0
            .model()
            .qigui523_game()
            .and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.id == message.source)
            })
            .map(|player| player.name.clone())
            .or_else(|| {
                client
                    .0
                    .model()
                    .texas_holdem_game()
                    .and_then(|game| {
                        game.players
                            .iter()
                            .find(|player| player.id == message.source)
                    })
                    .map(|player| player.name.clone())
            })
            .or_else(|| {
                client
                    .0
                    .model()
                    .shengji_game()
                    .and_then(|game| {
                        game.players
                            .iter()
                            .find(|player| player.id == message.source)
                    })
                    .map(|player| player.name.clone())
            })
            .unwrap_or_else(|| "玩家".to_owned());
        let (text, emoji) = match message.content {
            ChatContent::Text(text) => (text, None),
            ChatContent::QuickVoice(index) => {
                let Some(text) = QUICK_VOICES.get(usize::from(index)) else {
                    continue;
                };
                if let Some(sound) = assets.audio.quick_voice_sounds.get(usize::from(index)) {
                    commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
                }
                ((*text).to_owned(), None)
            }
            ChatContent::Emoji(emoji) => ("发送了表情".to_owned(), Some(emoji)),
        };
        chat.history.push_back(ChatHistoryEntry {
            player_name,
            message: text.clone(),
        });
        while chat.history.len() > CHAT_HISTORY_LIMIT {
            chat.history.pop_front();
        }

        let Some((layer, layer_node, layer_transform)) = layer else {
            continue;
        };
        let Some(anchor) =
            interaction_anchor_in_layer(source, layer_node, layer_transform, &avatars)
        else {
            continue;
        };
        for (entity, bubble) in &existing_bubbles {
            if bubble.player == source {
                commands.entity(entity).despawn();
            }
        }
        if let Some(previous) = spawned.remove(&source) {
            commands.entity(previous).despawn();
        }
        let entity = if let Some(emoji) = emoji {
            spawn_emoji_bubble(
                &mut commands,
                layer,
                layer_node.size() * layer_node.inverse_scale_factor(),
                anchor,
                source,
                emoji,
                &assets,
            )
        } else {
            spawn_chat_bubble(
                &mut commands,
                layer,
                layer_node.size() * layer_node.inverse_scale_factor(),
                anchor,
                source,
                &text,
                &assets,
            )
        };
        spawned.insert(source, entity);
    }
}

pub fn chat_bubble_position(anchor: Vec2, layer_size: Vec2, width: f32) -> Vec2 {
    let x = if anchor.x < layer_size.x * 0.34 {
        anchor.x + 27.0
    } else if anchor.x > layer_size.x * 0.66 {
        anchor.x - width - 27.0
    } else {
        anchor.x - width * 0.5
    };
    Vec2::new(
        x.clamp(8.0, (layer_size.x - width - 8.0).max(8.0)),
        (anchor.y - 69.0).clamp(8.0, (layer_size.y - 48.0).max(8.0)),
    )
}

fn spawn_chat_bubble(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    anchor: Vec2,
    player: PlayerId,
    message: &str,
    assets: &UiAssets,
) -> Entity {
    let character_count = message.chars().count();
    let width = (character_count as f32 * 13.0 + 26.0).clamp(76.0, 230.0);
    let position = chat_bubble_position(anchor, layer_size, width);
    let bubble = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                width: px(width),
                min_width: px(width),
                max_width: px(width),
                min_height: px(38),
                padding: UiRect::axes(px(11), px(7)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(9)),
                ..default()
            },
            BackgroundColor(PANEL.with_alpha(0.0)),
            BorderColor::all(ACCENT.with_alpha(0.0)),
            UiTransform {
                translation: Val2::px(0.0, 9.0),
                scale: Vec2::splat(0.88),
                ..UiTransform::IDENTITY
            },
            // 高于自己的常驻得分框（1500），避免左下角气泡被遮住。
            GlobalZIndex(1600),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(bubble);
    let text = add_text(commands, bubble, message, 13.0, Color::NONE, assets);
    commands.entity(text).insert(ChatBubbleText);
    commands.entity(bubble).insert(ActiveChatBubble {
        player,
        text: Some(text),
        emoji_image: None,
        emoji: false,
        width,
        elapsed: 0.0,
        duration: 2.8 + (character_count as f32 * 0.055).min(2.2),
    });
    bubble
}

fn spawn_emoji_bubble(
    commands: &mut Commands,
    layer: Entity,
    layer_size: Vec2,
    anchor: Vec2,
    player: PlayerId,
    emoji: ChatEmoji,
    assets: &UiAssets,
) -> Entity {
    let width = 78.0;
    let position = chat_bubble_position(anchor, layer_size, width);
    let bubble = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(position.x),
                top: px(position.y),
                width: px(width),
                height: px(width),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            UiTransform::IDENTITY,
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            GlobalZIndex(1600),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(layer).add_child(bubble);
    let image = assets.chat_emoji(emoji);
    let icon = commands
        .spawn((
            Node {
                width: px(72),
                height: px(72),
                ..default()
            },
            ImageNode::new(image),
            FocusPolicy::Pass,
        ))
        .id();
    commands.entity(bubble).add_child(icon);
    commands.entity(bubble).insert(ActiveChatBubble {
        player,
        text: None,
        emoji_image: Some(icon),
        emoji: true,
        width,
        elapsed: 0.0,
        duration: 3.4,
    });
    bubble
}

pub fn animate_chat_bubbles(
    time: Res<Time>,
    mut commands: Commands,
    layers: Query<(&ComputedNode, &UiGlobalTransform), With<PlayerInteractionLayer>>,
    avatars: Query<(&PlayerAvatarAnchor, &ComputedNode, &UiGlobalTransform)>,
    mut bubbles: Query<(
        Entity,
        &mut ActiveChatBubble,
        &mut Node,
        &mut UiTransform,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut texts: Query<&mut TextColor, With<ChatBubbleText>>,
    mut emoji_images: Query<&mut ImageNode>,
) {
    let Ok((layer_node, layer_transform)) = layers.single() else {
        return;
    };
    let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
    for (entity, mut bubble, mut node, mut transform, mut background, mut border) in &mut bubbles {
        bubble.elapsed += time.delta_secs();
        if bubble.elapsed >= bubble.duration {
            commands.entity(entity).despawn();
            continue;
        }
        let Some(anchor) =
            interaction_anchor_in_layer(bubble.player, layer_node, layer_transform, &avatars)
        else {
            commands.entity(entity).despawn();
            continue;
        };
        let position = chat_bubble_position(anchor, layer_size, bubble.width);
        node.left = px(position.x);
        node.top = px(position.y);

        let enter = ease_out_cubic((bubble.elapsed / 0.18).clamp(0.0, 1.0));
        let fade = ((bubble.duration - bubble.elapsed) / 0.48).clamp(0.0, 1.0);
        let alpha = enter * fade;
        transform.translation = Val2::px(0.0, 9.0 * (1.0 - enter) - bubble.elapsed * 1.4);
        transform.scale = Vec2::splat(if bubble.emoji {
            0.84 + enter * 0.16
        } else {
            0.88 + enter * 0.12
        });
        background.0 = if bubble.emoji {
            Color::NONE
        } else {
            PANEL.with_alpha(0.96 * alpha)
        };
        border.set_all(if bubble.emoji {
            Color::NONE
        } else {
            ACCENT.with_alpha(0.78 * alpha)
        });
        if let Some(text) = bubble.text
            && let Ok(mut color) = texts.get_mut(text)
        {
            color.0 = TEXT.with_alpha(alpha);
        }
        if let Some(image) = bubble.emoji_image
            && let Ok(mut image) = emoji_images.get_mut(image)
        {
            image.color = Color::WHITE.with_alpha(alpha);
        }
    }
}

pub fn animate_chat_panel(
    time: Res<Time>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    mut panels: Query<&mut UiTransform, With<ChatPanel>>,
    mut icons: Query<&mut ImageNode, With<ChatToggleIcon>>,
) {
    let target = if chat.open { 0.0 } else { 1.0 };
    if (chat.slide - target).abs() < 0.001 {
        if chat.slide != target {
            chat.slide = target;
            for mut transform in &mut panels {
                transform.translation = chat_panel_translation(target);
            }
        } else {
            let expected = chat_panel_translation(target);
            for mut transform in &mut panels {
                if transform.translation != expected {
                    transform.translation = expected;
                }
            }
        }
        return;
    }
    let response = 1.0 - (-time.delta_secs() * 12.0).exp();
    chat.slide += (target - chat.slide) * response;
    if (chat.slide - target).abs() < 0.001 {
        chat.slide = target;
    }
    for mut transform in &mut panels {
        transform.translation = chat_panel_translation(chat.slide);
    }
    let expected = if chat.open {
        &assets.social.chat_close_icon
    } else {
        &assets.social.chat_open_icon
    };
    for mut icon in &mut icons {
        if icon.image != *expected {
            icon.image = expected.clone();
        }
    }
}

fn chat_panel_translation(slide: f32) -> Val2 {
    Val2::px(CHAT_PANEL_HIDDEN_OFFSET * slide, 0.0)
}

pub fn animate_auto_play_robot_indicators(
    time: Res<Time>,
    mut lights: Query<(
        &AutoPlayAntennaLight,
        &mut UiTransform,
        &mut BackgroundColor,
    )>,
) {
    for (light, mut transform, mut background) in &mut lights {
        let phase = time.elapsed_secs() * 3.0 + f32::from(light.player.0) * 0.61;
        let pulse = (phase.sin() + 1.0) * 0.5;
        match light.part {
            AutoPlayAntennaLightPart::Glow => {
                transform.scale = Vec2::splat(0.72 + pulse * 0.58);
                background.0 = Color::srgba(0.32, 1.0, 0.58, 0.08 + pulse * 0.54);
            }
            AutoPlayAntennaLightPart::Ray => {
                transform.scale = Vec2::new(1.0, 0.68 + pulse * 0.42);
                background.0 = Color::srgba(0.46, 1.0, 0.68, 0.04 + pulse * 0.82);
            }
        }
    }
}

pub fn sync_chat_panel_text(
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

pub fn chat_input_display(input: &str) -> String {
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

pub fn scroll_chat_menus(
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
        for (cursor, mut position, node) in &mut quick_voice_scrolls {
            if !cursor.cursor_over() {
                continue;
            }
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            let next = (position.y - delta).clamp(0.0, maximum);
            position.y = next;
            chat.quick_voice_scroll_y = next;
        }
    }
    if chat.emoji_open {
        for (cursor, mut position, node) in &mut emoji_scrolls {
            if !cursor.cursor_over() {
                continue;
            }
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
            let next = (position.y - delta).clamp(0.0, maximum);
            position.y = next;
            chat.emoji_scroll_y = next;
        }
    }
}
