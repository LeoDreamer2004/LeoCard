use leocard_protocol::{ChatMessage, PlayerInteraction};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub(crate) struct EventInbox<E>(VecDeque<E>);

impl<E> Default for EventInbox<E> {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl<E> EventInbox<E> {
    pub(crate) fn push(&mut self, event: E) {
        self.0.push_back(event);
    }

    pub(crate) fn take(&mut self) -> Vec<E> {
        self.0.drain(..).collect()
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EventSequence<T> {
    pub(crate) value: Option<T>,
    pub(crate) serial: u64,
}

impl<T> Default for EventSequence<T> {
    fn default() -> Self {
        Self {
            value: None,
            serial: 0,
        }
    }
}

impl<T> EventSequence<T> {
    pub(crate) fn publish(&mut self, value: T) {
        self.serial = self.serial.saturating_add(1);
        self.value = Some(value);
    }

    pub(crate) fn clear(&mut self) {
        self.value = None;
        self.serial = 0;
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct PendingEvents {
    pub(super) player_interactions: EventInbox<PlayerInteraction>,
    pub(super) chat_messages: EventInbox<ChatMessage>,
}

impl PendingEvents {
    pub(super) fn clear(&mut self) {
        self.player_interactions.clear();
        self.chat_messages.clear();
    }
}
