use crate::RoomSession;
use leocard_draw_guess::{DrawGuessGame, DrawGuessPhase};
use leocard_game_common::PlayerId;

pub(super) struct DrawGuessBots {
    players: Vec<PlayerId>,
}

impl DrawGuessBots {
    const GUESS_REMAINING_MS: u32 = 5000;

    pub(super) fn new(room: &RoomSession) -> Self {
        Self {
            players: room
                .players
                .iter()
                .filter(|player| player.is_bot && !player.left)
                .map(|player| player.id)
                .collect(),
        }
    }

    pub(super) fn advance(&self, game: &mut DrawGuessGame, mut elapsed: u32) -> bool {
        let mut changed = false;
        while game.phase() != DrawGuessPhase::Finished {
            changed |= self.guess(game);
            if elapsed == 0 {
                break;
            }
            // Stop at each phase boundary and guessing deadline, including large clock ticks.
            let until_action = if self.pending_guesser(game).is_some() {
                game.remaining_ms().saturating_sub(Self::GUESS_REMAINING_MS)
            } else {
                game.remaining_ms()
            };
            let step = elapsed.min(until_action);
            changed |= game.advance_ms(step);
            elapsed -= step;
        }
        changed
    }

    fn pending_guesser(&self, game: &DrawGuessGame) -> Option<PlayerId> {
        if game.phase() != DrawGuessPhase::Drawing {
            return None;
        }
        game.players()
            .iter()
            .find(|player| {
                self.players.contains(&player.id)
                    && player.connected
                    && !player.guessed
                    && game.artist() != Some(player.id)
            })
            .map(|player| player.id)
    }

    fn guess(&self, game: &mut DrawGuessGame) -> bool {
        if game.remaining_ms() > Self::GUESS_REMAINING_MS || self.pending_guesser(game).is_none() {
            return false;
        }
        let answer = game
            .answer()
            .expect("drawing has a selected word")
            .to_owned();
        while let Some(player) = self.pending_guesser(game) {
            game.guess(player, &answer)
                .expect("connected bot submits its first guess of the turn");
        }
        true
    }
}
