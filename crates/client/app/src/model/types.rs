use leocard_protocol::{ChatMessage, PlayerInteraction};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub(super) struct GameEventInbox<E>(VecDeque<E>);

impl<E> Default for GameEventInbox<E> {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl<E> GameEventInbox<E> {
    pub(super) fn push(&mut self, event: E) {
        self.0.push_back(event);
    }

    pub(super) fn take(&mut self) -> Vec<E> {
        self.0.drain(..).collect()
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Debug)]
pub(super) struct Sequenced<T> {
    pub(super) value: Option<T>,
    pub(super) serial: u64,
}

impl<T> Default for Sequenced<T> {
    fn default() -> Self {
        Self {
            value: None,
            serial: 0,
        }
    }
}

impl<T> Sequenced<T> {
    pub(super) fn publish(&mut self, value: T) {
        self.serial = self.serial.saturating_add(1);
        self.value = Some(value);
    }

    pub(super) fn clear(&mut self) {
        self.value = None;
        self.serial = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct PendingEvents {
    pub(super) player_interactions: GameEventInbox<PlayerInteraction>,
    pub(super) chat_messages: GameEventInbox<ChatMessage>,
}

impl PendingEvents {
    pub(super) fn clear(&mut self) {
        self.player_interactions.clear();
        self.chat_messages.clear();
    }
}
