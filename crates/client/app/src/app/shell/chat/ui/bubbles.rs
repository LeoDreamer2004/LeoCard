use super::super::{ChatBubbleOwner, ChatHistoryEntry, ChatPanelState, QUICK_VOICES};
use crate::app::presentation::{TEXT, card_background_image};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{PlayerAvatar, PlayerInteractionLayer, interaction_anchor_entity};
use bevy::prelude::*;
use leocard_game_common::PlayerId;
use leocard_protocol::ChatContent;
use leocard_ui::{AnchoredBubble, Bubble, BubbleContent, BubbleStyle, UiComposer};
use std::collections::HashMap;

pub(crate) fn sync_chat_messages(
    mut commands: UiComposer,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    layers: Query<Entity, With<PlayerInteractionLayer>>,
    avatars: Query<PlayerAvatar>,
    existing_bubbles: Query<(Entity, &ChatBubbleOwner)>,
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
            .player_name(source)
            .unwrap_or("玩家")
            .to_owned();
        let (text, emoji) = match message.content {
            ChatContent::Text(text) => (text, None),
            ChatContent::QuickVoice(index) => {
                let Some(text) = QUICK_VOICES.get(usize::from(index)) else {
                    continue;
                };
                if let Some(profile) = client.0.model().player_game_profiles(source)
                    && let Some(sound) = assets.audio.quick_voices.get(profile.gender, index)
                {
                    commands.effect((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
                }
                ((*text).to_owned(), None)
            }
            ChatContent::Emoji(emoji) => ("发送了表情".to_owned(), Some(emoji)),
        };
        chat.history.push_back(ChatHistoryEntry {
            player_name,
            message: text.clone(),
        });
        while chat.history.len() > 60 {
            chat.history.pop_front();
        }
        let Some(layer) = layer else {
            continue;
        };
        let Some(anchor) = interaction_anchor_entity(source, &avatars) else {
            continue;
        };
        for (entity, owner) in &existing_bubbles {
            if owner.0 == source {
                commands.entity(entity).despawn();
            }
        }
        if let Some(previous) = spawned.remove(&source) {
            commands.entity(previous).despawn();
        }
        let mut style = BubbleStyle::new(
            card_background_image(assets.home.game_card.clone(), 0.42),
            assets.font.clone(),
            TEXT,
        );
        style.anchor_offset = Vec2::new(0.0, 0.16);
        style.z_index = 1600;
        let (content, duration) = match emoji {
            Some(emoji) => (BubbleContent::Image(assets.chat_emoji(emoji)), 3.4),
            None => (
                BubbleContent::Text(text.as_str()),
                2.8 + (text.chars().count() as f32 * 0.055).min(2.2),
            ),
        };
        let entity = Bubble {
            anchor,
            content,
            duration,
        }
        .spawn(&mut commands, layer, &style);
        commands.entity(entity).insert(ChatBubbleOwner(source));
        spawned.insert(source, entity);
    }
}

/// Player identities are resolved here; the UI crate only knows anchor entities.
pub(crate) fn sync_chat_bubble_anchors(
    avatars: Query<PlayerAvatar>,
    mut bubbles: Query<(&ChatBubbleOwner, &mut AnchoredBubble)>,
) {
    for (owner, mut bubble) in &mut bubbles {
        bubble.anchor = interaction_anchor_entity(owner.0, &avatars);
    }
}
