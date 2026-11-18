use super::{DrawGuessGame, DrawGuessPhase, GuessMessage};
use crate::{DrawAction, DrawGuessError};
use leocard_game_common::PlayerId;

impl DrawGuessGame {
    pub fn choose_word(&mut self, player: PlayerId, choice: u8) -> Result<(), DrawGuessError> {
        self.require_artist(player)?;
        if self.phase != DrawGuessPhase::Choosing {
            return Err(DrawGuessError::WrongPhase);
        }
        let Some(word) = self.choices().get(usize::from(choice)).cloned() else {
            return Err(DrawGuessError::InvalidWords);
        };
        self.secret = Some(word);
        self.phase = DrawGuessPhase::Drawing;
        self.remaining_ms = u32::from(self.rules.drawing_seconds) * 1000;
        Ok(())
    }

    pub fn draw(&mut self, player: PlayerId, action: &DrawAction) -> Result<(), DrawGuessError> {
        self.require_artist(player)?;
        if self.phase != DrawGuessPhase::Drawing {
            return Err(DrawGuessError::WrongPhase);
        }
        self.canvas.apply(action)
    }

    pub fn guess(&mut self, player: PlayerId, text: &str) -> Result<GuessMessage, DrawGuessError> {
        if self.phase != DrawGuessPhase::Drawing {
            return Err(DrawGuessError::WrongPhase);
        }
        let index = self
            .players
            .iter()
            .position(|candidate| candidate.id == player)
            .ok_or(DrawGuessError::CannotGuess)?;
        if self.artist() == Some(player) || !self.players[index].connected {
            return Err(DrawGuessError::CannotGuess);
        }
        let private = self.players[index].guessed;
        let text = text.trim();
        if text.is_empty()
            || text.chars().count() > GuessMessage::MAX_CHARACTERS
            || text.chars().any(char::is_control)
            || (!private && Self::normalize(text).is_empty())
        {
            return Err(DrawGuessError::InvalidGuess);
        }
        if self.last_guess_ms[index]
            .is_some_and(|previous| self.elapsed_ms.saturating_sub(previous) < 350)
        {
            return Err(DrawGuessError::GuessTooSoon);
        }
        self.last_guess_ms[index] = Some(self.elapsed_ms);
        let correct =
            !private && Self::normalize(text) == Self::normalize(self.secret.as_deref().unwrap());
        let points = if correct {
            20 + 80 * self.remaining_ms / (u32::from(self.rules.drawing_seconds) * 1000)
        } else {
            0
        };
        if correct {
            self.players[index].guessed = true;
            self.players[index].score += points;
            let artist = self.turn as usize % self.players.len();
            self.players[artist].score += 50;
        }
        self.message_serial += 1;
        let message = GuessMessage {
            serial: self.message_serial,
            player,
            text: if correct {
                String::new()
            } else {
                text.to_owned()
            },
            correct,
            points,
            private,
        };
        self.history.push(message.clone());
        if self.history.len() > 128 {
            self.history.remove(0);
        }
        self.finish_if_everyone_guessed();
        Ok(message)
    }

    pub fn set_connected(&mut self, player: PlayerId, connected: bool) {
        if let Some(participant) = self
            .players
            .iter_mut()
            .find(|candidate| candidate.id == player)
        {
            participant.connected = connected;
        }
        if !connected && self.artist() == Some(player) {
            self.reveal();
        } else {
            self.finish_if_everyone_guessed();
        }
    }

    fn finish_if_everyone_guessed(&mut self) {
        if self.phase == DrawGuessPhase::Drawing
            && self.players.iter().all(|candidate| {
                Some(candidate.id) == self.artist() || candidate.guessed || !candidate.connected
            })
        {
            self.reveal();
        }
    }
}
