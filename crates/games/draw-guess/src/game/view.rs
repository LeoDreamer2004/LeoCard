use super::{DrawGuessGame, DrawGuessPhase, GuessMessage, GuessPlayer};
use crate::CanvasDocument;
use leocard_game_common::PlayerId;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawGuessView {
    pub phase: DrawGuessPhase,
    pub artist: Option<PlayerId>,
    pub turn: u32,
    pub total_turns: u32,
    pub remaining_ms: u32,
    pub players: Vec<GuessPlayer>,
    pub choices: Vec<String>,
    pub word: Option<String>,
    pub word_length: u16,
    pub canvas: CanvasDocument,
    pub history: Vec<GuessMessage>,
}

impl DrawGuessGame {
    pub fn view(&self, recipient: PlayerId) -> DrawGuessView {
        let artist = self.artist() == Some(recipient);
        let revealed = matches!(
            self.phase,
            DrawGuessPhase::Reveal | DrawGuessPhase::Finished
        );
        let guessed = self
            .players
            .iter()
            .any(|player| player.id == recipient && player.guessed);
        DrawGuessView {
            phase: self.phase,
            artist: self.artist(),
            turn: self.turn,
            total_turns: self.total_turns(),
            remaining_ms: self.remaining_ms,
            players: self.players.clone(),
            choices: if artist && self.phase == DrawGuessPhase::Choosing {
                self.choices()
            } else {
                Vec::new()
            },
            word: (artist || revealed || guessed)
                .then(|| self.secret.clone())
                .flatten(),
            word_length: self
                .secret
                .as_ref()
                .map_or(0, |word| word.chars().count() as u16),
            canvas: self.canvas.clone(),
            history: self
                .history
                .iter()
                .filter(|message| !message.private || artist || guessed)
                .cloned()
                .collect(),
        }
    }
}
