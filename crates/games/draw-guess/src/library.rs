use crate::{DrawGuessError, DrawGuessGame, GuessMessage, word_bank};
use std::collections::HashSet;

/// A portable word library. Local filenames and filesystem paths never cross the network.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WordLibrary {
    pub title: String,
    pub words: Vec<String>,
}

impl WordLibrary {
    pub const MAX_WORDS: usize = 1024;
    pub const MAX_WORD_CHARACTERS: usize = GuessMessage::MAX_CHARACTERS;
    pub const MAX_TITLE_CHARACTERS: usize = 40;
    pub const MAX_CONTENT_BYTES: usize = 65_536;

    pub fn builtin() -> Self {
        Self {
            title: "默认词库".into(),
            words: word_bank(),
        }
    }

    pub fn from_lines(title: &str, content: &str) -> Result<Self, DrawGuessError> {
        if content.len() > Self::MAX_CONTENT_BYTES {
            return Err(DrawGuessError::InvalidWordLibrary);
        }
        Self {
            title: title.to_owned(),
            words: content.lines().map(str::to_owned).collect(),
        }
        .validate()
    }

    pub fn validate(self) -> Result<Self, DrawGuessError> {
        let title = self.title.trim().to_owned();
        if title.is_empty()
            || title.chars().count() > Self::MAX_TITLE_CHARACTERS
            || title.chars().any(char::is_control)
            || self.words.len() > Self::MAX_WORDS
            || self.words.iter().map(String::len).sum::<usize>() > Self::MAX_CONTENT_BYTES
        {
            return Err(DrawGuessError::InvalidWordLibrary);
        }
        let mut seen = HashSet::new();
        let mut words = Vec::new();
        for word in self.words {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }
            let normalized = DrawGuessGame::normalize(word);
            if normalized.is_empty()
                || word.chars().count() > Self::MAX_WORD_CHARACTERS
                || word.chars().any(char::is_control)
            {
                return Err(DrawGuessError::InvalidWordLibrary);
            }
            if seen.insert(normalized) {
                words.push(word.to_owned());
            }
        }
        if words.len() < 3 {
            return Err(DrawGuessError::InvalidWords);
        }
        Ok(Self { title, words })
    }

    pub fn fingerprint(&self) -> u64 {
        self.title
            .bytes()
            .chain([0])
            .chain(self.words.iter().flat_map(|word| word.bytes().chain([0])))
            .fold(0xcbf29ce484222325, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            })
    }

    pub fn content(&self) -> String {
        self.words.join("\n")
    }
}
