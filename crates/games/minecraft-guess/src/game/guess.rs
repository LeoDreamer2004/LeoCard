use crate::{AttributeFeedback, Entry, MatchQuality};
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

    pub(super) fn all_matching(&self) -> bool {
        matches!(self, Self::Known { matches, .. } if !matches.is_empty()
            && matches.iter().all(|feedback| feedback.quality == MatchQuality::Exact))
    }

    pub fn has_entry(&self, id: &str) -> bool {
        matches!(self, Self::Known { entry, .. } if entry.id == id)
    }

    /// Unknown guesses reveal no attributes and cannot exclude a candidate.
    pub fn accepts_candidate(&self, candidate: &Entry) -> bool {
        self.accepts_candidate_matching(candidate, |_| true)
    }

    /// Use only selected public attributes while still excluding previously guessed entries.
    pub fn accepts_candidate_matching(
        &self,
        candidate: &Entry,
        include_attribute: impl Fn(&str) -> bool,
    ) -> bool {
        match self {
            Self::Unknown { .. } => true,
            Self::Known { entry, matches, .. } => {
                entry.id != candidate.id
                    && entry.attributes.len() == candidate.attributes.len()
                    && entry.attributes.len() == matches.len()
                    && entry
                        .attributes
                        .iter()
                        .zip(&candidate.attributes)
                        .zip(matches)
                        .filter(|((ours, _), _)| include_attribute(&ours.label))
                        .all(|((ours, theirs), feedback)| ours.feedback(theirs) == *feedback)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GuessRecord {
    pub player: u8,
    pub result: GuessResult,
}
