use crate::{CanvasDocument, DrawGuessError, DrawGuessRuleSet, WordLibrary};
use leocard_game_common::PlayerId;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawGuessPhase {
    Choosing,
    Drawing,
    Reveal,
    Finished,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuessPlayer {
    pub id: PlayerId,
    pub score: u32,
    pub guessed: bool,
    pub connected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuessMessage {
    pub serial: u32,
    pub player: PlayerId,
    /// Correct guesses never include the answer in the public history.
    pub text: String,
    pub correct: bool,
    pub points: u32,
    /// Post-solve chat is visible only to the artist and players who have guessed correctly.
    pub private: bool,
}

impl GuessMessage {
    pub const MAX_CHARACTERS: usize = 20;
}

#[derive(Clone, Debug)]
pub struct DrawGuessGame {
    pub(super) rules: DrawGuessRuleSet,
    pub(super) players: Vec<GuessPlayer>,
    pub(super) words: Vec<String>,
    pub(super) turn: u32,
    pub(super) phase: DrawGuessPhase,
    pub(super) remaining_ms: u32,
    pub(super) elapsed_ms: u64,
    pub(super) secret: Option<String>,
    pub(super) canvas: CanvasDocument,
    pub(super) history: Vec<GuessMessage>,
    pub(super) message_serial: u32,
    pub(super) last_guess_ms: Vec<Option<u64>>,
}

impl DrawGuessGame {
    pub fn new(
        rules: DrawGuessRuleSet,
        players: Vec<PlayerId>,
        words: Vec<String>,
    ) -> Result<Self, DrawGuessError> {
        let rules = rules.validate()?;
        if !(usize::from(DrawGuessRuleSet::MIN_PLAYERS)
            ..=usize::from(DrawGuessRuleSet::MAX_PLAYERS))
            .contains(&players.len())
            || players.iter().collect::<HashSet<_>>().len() != players.len()
        {
            return Err(DrawGuessError::InvalidPlayers);
        }
        let words: Vec<_> = words
            .into_iter()
            .map(|word| word.trim().to_owned())
            .filter(|word| {
                !Self::normalize(word).is_empty()
                    && word.chars().count() <= WordLibrary::MAX_WORD_CHARACTERS
                    && !word.chars().any(char::is_control)
            })
            .collect();
        if words
            .iter()
            .map(|word| Self::normalize(word))
            .collect::<HashSet<_>>()
            .len()
            < 3
        {
            return Err(DrawGuessError::InvalidWords);
        }
        let mut unique = HashSet::new();
        let words = words
            .into_iter()
            .filter(|word| unique.insert(Self::normalize(word)))
            .collect();
        let count = players.len();
        Ok(Self {
            rules,
            players: players
                .into_iter()
                .map(|id| GuessPlayer {
                    id,
                    score: 0,
                    guessed: false,
                    connected: true,
                })
                .collect(),
            words,
            turn: 0,
            phase: DrawGuessPhase::Choosing,
            remaining_ms: u32::from(DrawGuessRuleSet::CHOICE_SECONDS) * 1000,
            elapsed_ms: 0,
            secret: None,
            canvas: CanvasDocument::default(),
            history: Vec::new(),
            message_serial: 0,
            last_guess_ms: vec![None; count],
        })
    }

    pub fn phase(&self) -> DrawGuessPhase {
        self.phase
    }

    pub fn turn(&self) -> u32 {
        self.turn
    }

    pub fn remaining_ms(&self) -> u32 {
        self.remaining_ms
    }

    pub fn players(&self) -> &[GuessPlayer] {
        &self.players
    }

    /// The authoritative answer. Network recipients must use the filtered view instead.
    pub fn answer(&self) -> Option<&str> {
        self.secret.as_deref()
    }

    pub fn total_turns(&self) -> u32 {
        self.players.len() as u32 * u32::from(self.rules.rounds)
    }

    pub fn artist(&self) -> Option<PlayerId> {
        (self.phase != DrawGuessPhase::Finished)
            .then(|| self.players[self.turn as usize % self.players.len()].id)
    }

    pub fn canvas(&self) -> &CanvasDocument {
        &self.canvas
    }

    pub(super) fn choices(&self) -> Vec<String> {
        (0..3)
            .map(|offset| self.words[(self.turn as usize * 3 + offset) % self.words.len()].clone())
            .collect()
    }

    pub(super) fn require_artist(&self, player: PlayerId) -> Result<(), DrawGuessError> {
        if self.artist() != Some(player) {
            return Err(DrawGuessError::NotArtist);
        }
        Ok(())
    }

    pub(crate) fn normalize(text: &str) -> String {
        text.chars()
            .filter_map(|character| {
                let character = match character {
                    '\u{FF01}'..='\u{FF5E}' => char::from_u32(character as u32 - 0xFEE0).unwrap(),
                    other => other,
                };
                (!character.is_whitespace() && !character.is_ascii_punctuation())
                    .then_some(character)
            })
            .flat_map(char::to_lowercase)
            .collect()
    }
}
