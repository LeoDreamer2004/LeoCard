use leocard_protocol::{PlayerId, PlayerInteraction, PlayerInteractionKind};
use std::{collections::VecDeque, time::Duration};

#[derive(Clone, Copy, Debug)]
pub enum PersonalEvent {
    AvatarSaved,
    TableBackgroundChanged,
    ClientUpdated,
    InteractionRun {
        kind: PlayerInteractionKind,
        count: u8,
        span: Duration,
    },
}

#[derive(Default)]
pub struct PersonalActivity {
    target: Option<PlayerId>,
    shoes: VecDeque<Duration>,
}

impl PersonalActivity {
    /// Only the local sender's accepted actions participate in a consecutive run.
    pub fn observe_interaction(
        &mut self,
        player: PlayerId,
        event: &PlayerInteraction,
    ) -> Option<PersonalEvent> {
        if event.source != player {
            return None;
        }
        if event.kind != PlayerInteractionKind::Shoe || self.target != Some(event.target) {
            self.shoes.clear();
            self.target = None;
        }
        if event.kind != PlayerInteractionKind::Shoe {
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
