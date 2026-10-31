use crate::AchievementTrigger;
use leocard_game_common::PlayerId;
use std::any::Any;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// An owned, local fact. Its concrete type and interpretation belong to its game.
/// This envelope is never serialized or transmitted over the network.
#[derive(Clone, Default)]
pub struct GameFacts(Option<Arc<dyn Any + Send + Sync>>);

impl GameFacts {
    pub fn new<T: Any + Send + Sync>(event: T) -> Self {
        Self(Some(Arc::new(event)))
    }
}

impl Debug for GameFacts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameFacts").finish_non_exhaustive()
    }
}

impl AchievementTrigger {
    /// Only the owner of T interprets its payload; unrelated games receive None.
    pub fn game<T: Any>(&self) -> Option<(PlayerId, &T)> {
        let Self::Game { player, event } = self else {
            return None;
        };
        Some((*player, event.0.as_ref()?.downcast_ref()?))
    }
}
