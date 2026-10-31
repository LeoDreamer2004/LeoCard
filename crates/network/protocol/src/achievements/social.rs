use crate::{ChatContent, ChatMessage, PlayerInteraction, PlayerInteractionKind};
use leocard_achievements::{ChatActivity, ChatFacts, InteractionFacts, InteractionKind};

impl From<&PlayerInteraction> for InteractionFacts {
    fn from(event: &PlayerInteraction) -> Self {
        Self {
            source: event.source,
            target: event.target,
            elapsed_millis: event.elapsed_millis,
            kind: match event.kind {
                PlayerInteractionKind::Flower => InteractionKind::Flower,
                PlayerInteractionKind::Egg => InteractionKind::Egg,
                PlayerInteractionKind::Wine => InteractionKind::Wine,
                PlayerInteractionKind::Shoe => InteractionKind::Shoe,
            },
        }
    }
}

impl From<&ChatMessage> for ChatFacts {
    fn from(message: &ChatMessage) -> Self {
        Self {
            source: message.source,
            content: match &message.content {
                ChatContent::Text(text) => ChatActivity::Text(text.clone()),
                ChatContent::QuickVoice(_) => ChatActivity::QuickVoice,
                ChatContent::Emoji(_) => ChatActivity::Other,
            },
        }
    }
}
