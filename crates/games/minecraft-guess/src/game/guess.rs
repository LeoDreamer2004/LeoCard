use crate::{AttributeFeedback, Catalogue, Entry, GameError, MAX_GUESS_CHARACTERS, MatchQuality};
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
    pub(crate) fn resolve(
        answer: &Entry,
        text: &str,
        allow_unknown: bool,
    ) -> Result<Self, GameError> {
        let name = text.trim();
        if name.is_empty() || name.chars().any(char::is_control) {
            return Err(GameError::InvalidGuess);
        }
        let entry = Catalogue::bundled().find(answer.kind, name);
        if !entry.is_some_and(|entry| entry.id == name)
            && name.chars().count() > MAX_GUESS_CHARACTERS
        {
            return Err(GameError::GuessTooLong);
        }
        match entry {
            Some(entry) => Ok(Self::Known {
                entry: entry.clone(),
                correct: entry.id == answer.id,
                matches: entry
                    .attributes
                    .iter()
                    .zip(&answer.attributes)
                    .map(|(ours, theirs)| ours.feedback(theirs))
                    .collect(),
            }),
            None if allow_unknown => Ok(Self::Unknown {
                name: name.to_owned(),
            }),
            None => Err(GameError::UnknownEntry),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Known { entry, .. } => &entry.name,
            Self::Unknown { name } => name,
        }
    }

    pub fn is_correct(&self) -> bool {
        matches!(self, Self::Known { correct: true, .. })
    }

    pub(crate) fn all_matching(&self) -> bool {
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
