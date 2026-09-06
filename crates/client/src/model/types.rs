use leocard_protocol::{ChatMessage, PlayerInteraction};
use std::collections::VecDeque;

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
    pub(super) player_interactions: VecDeque<PlayerInteraction>,
    pub(super) chat_messages: VecDeque<ChatMessage>,
}

impl PendingEvents {
    pub(super) fn clear(&mut self) {
        self.player_interactions.clear();
        self.chat_messages.clear();
    }
}
