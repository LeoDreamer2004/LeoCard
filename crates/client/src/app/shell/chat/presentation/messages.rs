use super::super::*;
use crate::app::presentation::{ACCENT, PANEL, add_text};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{PlayerAvatarAnchor, PlayerInteractionLayer, interaction_anchor_in_layer};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use leocard_protocol::{ChatContent, ChatEmoji, PlayerId};
use std::collections::HashMap;

const CHAT_HISTORY_LIMIT: usize = 60;

pub(crate) fn sync_chat_messages(
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
            .player_name(message.source)
            .unwrap_or("玩家")
            .to_owned();
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
        let layer_size = layer_node.size() * layer_node.inverse_scale_factor();
        let entity = if let Some(emoji) = emoji {
            spawn_emoji_bubble(
                &mut commands,
                layer,
                layer_size,
                anchor,
                source,
                emoji,
                &assets,
            )
        } else {
            spawn_chat_bubble(
                &mut commands,
                layer,
                layer_size,
                anchor,
                source,
                &text,
                &assets,
            )
        };
        spawned.insert(source, entity);
    }
}

pub(crate) fn chat_bubble_position(anchor: Vec2, layer_size: Vec2, width: f32) -> Vec2 {
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
    let icon = commands
        .spawn((
            Node {
                width: px(72),
                height: px(72),
                ..default()
            },
            ImageNode::new(assets.chat_emoji(emoji)),
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
