use crate::facts::{InteractionFacts, InteractionKind, ParticipantId};
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum PersonalEvent {
    AvatarSaved,
    TableBackgroundChanged,
    ClientUpdated,
    ItemPurchased {
        price: u32,
    },
    CoinBalance {
        coins: u32,
    },
    SakuraTunnelStarted,
    InteractionRun {
        kind: InteractionKind,
        count: u8,
        span: Duration,
    },
}

#[derive(Default)]
pub struct PersonalActivity {
    target: Option<ParticipantId>,
    shoes: VecDeque<Duration>,
}

impl PersonalActivity {
    /// Only the local sender's accepted actions participate in a consecutive run.
    pub fn observe_interaction(
        &mut self,
        player: ParticipantId,
        event: &InteractionFacts,
    ) -> Option<PersonalEvent> {
        if event.source != player {
            return None;
        }
        if event.kind != InteractionKind::Shoe || self.target != Some(event.target) {
            self.shoes.clear();
            self.target = None;
        }
        if event.kind != InteractionKind::Shoe {
            return None;
        }
        let now = Duration::from_millis(event.elapsed_millis);
        self.target = Some(event.target);
        self.shoes.push_back(now);
        while self.shoes.len() > 5
            || self
                .shoes
                .front()
                .is_some_and(|first| now.saturating_sub(*first) > Duration::from_secs(60))
        {
            self.shoes.pop_front();
        }
        Some(PersonalEvent::InteractionRun {
            kind: event.kind,
            count: self.shoes.len() as u8,
            span: now.saturating_sub(
                *self
                    .shoes
                    .front()
                    .expect("the current interaction remains in its window"),
            ),
        })
    }
}
