use super::super::state::ChatBubbleSlot;
use super::super::{ChatHistoryEntry, ChatPanelState, QUICK_VOICES};
use crate::app::presentation::{TEXT, card_background_image};
use crate::app::runtime::{ClientResource, UiAssets};
use crate::app::shell::{PlayerAvatar, PlayerInteractionLayer, interaction_anchor_entity};
use bevy::prelude::*;
use leocard_protocol::ChatContent;
use leocard_ui::{AnchoredBubble, Bubble, BubbleContent, BubblePlacement, BubbleStyle, UiComposer};
use std::collections::HashMap;

pub(crate) fn sync_chat_messages(
    mut commands: UiComposer,
    mut client: Option<ResMut<ClientResource>>,
    assets: Res<UiAssets>,
    mut chat: ResMut<ChatPanelState>,
    layers: Query<Entity, With<PlayerInteractionLayer>>,
    avatars: Query<PlayerAvatar>,
    existing_bubbles: Query<(Entity, &ChatBubbleSlot)>,
) {
    let Some(client) = client.as_deref_mut() else {
        return;
    };
    let messages = client.0.take_chat_messages();
    if messages.is_empty() {
        return;
    }
    let model = client.0.model();
    let layer = layers
        .single()
        .ok()
        .filter(|_| model.game_snapshot().is_some());
    let mut spawned = HashMap::<ChatBubbleSlot, Entity>::new();
    for message in messages {
        let source = message.source;
        let spectator_heading = message
            .spectator_name
            .map(|name| format!("旁观玩家【{}】：", name.replace(['\r', '\n'], " ")));
        let (text, emoji) = match message.content {
            ChatContent::Text(text) => (text, None),
            ChatContent::QuickVoice(index) => {
                let Some(text) = QUICK_VOICES.get(usize::from(index)) else {
                    continue;
                };
                if let Some(profile) = model.player_game_profiles(source)
                    && let Some(sound) = assets.audio.quick_voices.get(profile.gender, index)
                {
                    commands.effect((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
                }
                ((*text).to_owned(), None)
            }
            ChatContent::Emoji(emoji) => ("发送了表情".to_owned(), Some(emoji)),
        };
        chat.history.push_back(ChatHistoryEntry {
            sender_label: spectator_heading.clone().unwrap_or_else(|| {
                format!("[{}(玩家)]:", model.player_name(source).unwrap_or("玩家"))
            }),
            message: text.clone(),
        });
        while chat.history.len() > 60 {
            chat.history.pop_front();
        }
        let Some(layer) = layer else {
            continue;
        };
        let slot = ChatBubbleSlot(source);
        let anchor = interaction_anchor_entity(source, &avatars);
        let placement = if spectator_heading.is_some() {
            BubblePlacement::Beside(anchor)
        } else {
            BubblePlacement::Anchor(anchor)
        };
        if let Some(previous) = spawned.remove(&slot) {
            commands.entity(previous).despawn();
        } else {
            for (entity, existing_slot) in &existing_bubbles {
                if *existing_slot == slot {
                    commands.entity(entity).despawn();
                }
            }
        }
        let mut style = BubbleStyle::new(
            card_background_image(assets.home.game_card.clone(), 0.42),
            assets.font.clone(),
            TEXT,
        );
        style.anchor_offset = if spectator_heading.is_some() {
            Vec2::new(0.5, 0.0)
        } else {
            Vec2::new(0.0, 0.16)
        };
        style.z_index = 1600;
        let duration = 2.8 + (text.chars().count() as f32 * 0.055).min(2.2);
        let (content, duration) = match emoji {
            Some(emoji) => (BubbleContent::Image(assets.chat_emoji(emoji)), 3.4),
            None => (BubbleContent::Text(text.as_str()), duration),
        };
        let entity = Bubble {
            placement,
            content,
            duration,
        }
        .spawn(&mut commands, layer, &style);
        commands.entity(entity).insert(slot);
        spawned.insert(slot, entity);
    }
}

/// Player identities are resolved here; the UI crate only knows anchor entities.
pub(crate) fn sync_chat_bubble_anchors(
    mut commands: Commands,
    client: Option<Res<ClientResource>>,
    avatars: Query<PlayerAvatar>,
    mut bubbles: Query<(Entity, &ChatBubbleSlot, &mut AnchoredBubble)>,
) {
    let playing = client.is_some_and(|client| client.0.model().game_snapshot().is_some());
    for (entity, slot, mut bubble) in &mut bubbles {
        if !playing {
            commands.entity(entity).despawn();
        } else {
            let anchor = interaction_anchor_entity(slot.0, &avatars);
            bubble.placement = match bubble.placement {
                BubblePlacement::Beside(_) => BubblePlacement::Beside(anchor),
                _ => BubblePlacement::Anchor(anchor),
            };
        }
    }
}
