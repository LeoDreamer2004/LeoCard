use super::McGuessBotObservation;
use crate::McGuessCatalogue;
use fastrand::Rng;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum McGuessBotAction {
    Guess { node: Option<u16>, entry_id: String },
    Skip { node: Option<u16> },
}

impl McGuessBotAction {
    pub fn target_node(&self) -> Option<u16> {
        match *self {
            Self::Guess { node, .. } | Self::Skip { node } => node,
        }
    }
}

pub struct McGuessBot;

impl McGuessBot {
    pub fn choose(observation: &McGuessBotObservation, rng: &mut Rng) -> Option<McGuessBotAction> {
        let full_feedback = cfg!(debug_assertions) || cfg!(feature = "developer");
        let candidates: Vec<_> = McGuessCatalogue::bundled()
            .search(observation.kind, "")
            .filter(|entry| {
                (observation.character_count == 0
                    || entry.name.chars().count() == usize::from(observation.character_count))
                    && observation
                        .related_entries
                        .as_ref()
                        .is_none_or(|related| related.contains(&entry.id))
                    && observation.guesses.iter().all(|guess| {
                        guess.accepts_candidate_matching(entry, |label| {
                            full_feedback || !matches!(label, "硬度" | "爆炸抗性")
                        })
                    })
            })
            .collect();
        if candidates.is_empty() {
            return observation.can_skip.then_some(McGuessBotAction::Skip {
                node: observation.node,
            });
        }
        let entry = candidates[rng.usize(..candidates.len())];
        Some(McGuessBotAction::Guess {
            node: observation.node,
            entry_id: entry.id.clone(),
        })
    }
}
