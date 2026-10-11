use crate::{AttributeFeedback, Entry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum GuessResult {
    Known {
        entry: Entry,
        matches: Vec<AttributeFeedback>,
        correct: bool,
    },
    Unknown {
        name: String,
    },
}

impl GuessResult {
    pub fn name(&self) -> &str {
        match self {
            Self::Known { entry, .. } => &entry.name,
            Self::Unknown { name } => name,
        }
    }

    pub fn is_correct(&self) -> bool {
        matches!(self, Self::Known { correct: true, .. })
    }

    pub fn has_entry(&self, id: &str) -> bool {
        matches!(self, Self::Known { entry, .. } if entry.id == id)
    }

    /// Unknown guesses reveal no attributes and cannot exclude a candidate.
    pub fn accepts_candidate(&self, candidate: &Entry) -> bool {
        match self {
            Self::Unknown { .. } => true,
            Self::Known { entry, matches, .. } => {
                entry.id != candidate.id
                    && entry
                        .attributes
                        .iter()
                        .zip(&candidate.attributes)
                        .map(|(ours, theirs)| ours.feedback(theirs))
                        .eq(matches.iter().copied())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessRecord {
    pub player: u8,
    pub result: GuessResult,
}
