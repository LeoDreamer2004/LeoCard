use super::super::*;
use super::*;
use crate::app::presentation::{ACCENT, PANEL, TEXT, ease_out_cubic};
use crate::app::runtime::UiAssets;
use crate::app::shell::{PlayerAvatarAnchor, PlayerInteractionLayer, interaction_anchor_in_layer};
use bevy::prelude::*;

pub(crate) fn animate_chat_bubbles(
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

pub(crate) fn animate_chat_panel(
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
